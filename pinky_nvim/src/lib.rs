use std::{cell::RefCell, collections::BTreeMap, path::PathBuf, rc::Rc, str::FromStr};

use nvim_oxi::{
    api::{
        self,
        opts::{CreateAutocmdOpts, NotifyOpts, NotifyOptsBuilder},
    },
    mlua::{lua, Function as LuaFunction, IntoLuaMulti, String as LuaString, Table},
    print, Dictionary, Function, String as NvimString,
};
use pinky::library::Library;

#[nvim_oxi::plugin]
fn pinky_nvim() -> nvim_oxi::Result<Dictionary> {
    let library: Rc<RefCell<Option<Library>>> = Rc::default();
    let lua = lua();

    // TODO: handle multiple libraries in setup
    let setup_lib = library.clone();
    let setup = Function::from_fn(move |opts: Dictionary| match opts.get("default_lib") {
        Some(dl) => {
            let dl_string = unsafe { dl.clone().into_string_unchecked().to_string() };
            let dl_path = PathBuf::from_str(&dl_string).unwrap();

            let mut sl = setup_lib.borrow_mut();
            *sl = Some(Library::open(&dl_path));

            let sl = sl.as_ref().unwrap();

            // TODO: going to have to swap out autocmds when we change libs
            // NOTE: also we might want to adjust these path patterns at some
            // point to include more things

            let mut commit_page_pattern = sl.path.clone();
            commit_page_pattern.push("*.md");
            let commit_page_lib = setup_lib.clone();
            api::create_autocmd(
                ["BufWrite"],
                &CreateAutocmdOpts::builder()
                    .patterns([commit_page_pattern.to_str().unwrap()])
                    .callback(move |args: api::types::AutocmdCallbackArgs| {
                        let cpl = commit_page_lib.borrow();
                        if cpl.is_some() {
                            cpl.as_ref().unwrap().commit_page(args.file)
                        }
                        false
                    })
                    .build(),
            )
            .unwrap();

            let mut schema_path_pattern = sl.path.clone();
            schema_path_pattern.push(sl.config.schema_dir.clone());
            schema_path_pattern.push("*.yaml");
            let commit_schema_lib = setup_lib.clone();
            api::create_autocmd(
                ["BufWrite"],
                &CreateAutocmdOpts::builder()
                    .patterns([schema_path_pattern.to_str().unwrap()])
                    .callback(move |args: api::types::AutocmdCallbackArgs| {
                        let csl = commit_schema_lib.borrow();
                        if csl.is_some() {
                            match csl.as_ref().unwrap().commit_schema(args.file) {
                                Ok(_) => {}
                                Err(e) => print!("{e}"),
                            }
                        }
                        false
                    })
                    .build(),
            )
            .unwrap();
        }
        None => api::err_writeln("pinky error: no default lib enabled"),
    });

    let new_page_lib = library.clone();
    let new_page = Function::from_fn(move |opts: Dictionary| {
        let npl = new_page_lib.borrow();
        if npl.is_none() {
            api::err_writeln("pinky error: no library loaded");
            return;
        }

        let schema = match opts.get("schema") {
            Some(s) => Some(unsafe { s.clone().into_string_unchecked().to_string() }),
            None => None,
        };

        let lib = npl.as_ref().unwrap();
        let page_path = lib.new_page(schema);
        let page_path_string = page_path.to_str().unwrap().to_string();

        api::command(&format!("e {}", page_path_string)).unwrap();
    });

    let new_schema_lib = library.clone();
    let new_schema = Function::from_fn(move |opts: Dictionary| {
        let nsl = new_schema_lib.borrow();
        if nsl.is_none() {
            api::err_writeln("pinky error: no library loaded");
            return;
        }

        let vim_ui_input = lua
            .globals()
            .get::<_, Table>("vim")
            .unwrap()
            .get::<_, Table>("ui")
            .unwrap()
            .get::<_, LuaFunction>("input")
            .unwrap();

        let callback_lib = new_schema_lib.clone();
        let callback = lua
            .create_function(move |_, input: Option<LuaString>| {
                let cb_lib = callback_lib.borrow();
                let lib = cb_lib.as_ref().unwrap();
                let schema_path = lib.new_schema(input.unwrap().to_str().unwrap());
                let schema_path_string = schema_path.to_str().unwrap().to_string();

                api::command(&format!("e {}", schema_path_string)).unwrap();
                Ok(())
            })
            .unwrap();

        vim_ui_input
            .call::<(BTreeMap<&str, &str>, LuaFunction), ()>((
                BTreeMap::from_iter([("prompt", "schema name: ")]),
                callback,
            ))
            .unwrap();
    });

    let pinky_nvim = Dictionary::from_iter([
        ("setup", setup),
        ("new_page", new_page),
        ("new_schema", new_schema),
    ]);
    Ok(pinky_nvim)
}
