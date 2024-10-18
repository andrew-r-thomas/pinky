use std::{cell::RefCell, path::PathBuf, rc::Rc, str::FromStr};

use nvim_oxi::{
    api::{self},
    print, Dictionary, Function,
};
use pinky::library::Library;

#[nvim_oxi::plugin]
fn pinky_nvim() -> nvim_oxi::Result<Dictionary> {
    let library: Rc<RefCell<Option<Library>>> = Rc::default();

    // TODO: handle multiple libraries in setup
    let setup_lib = library.clone();
    let setup = Function::from_fn(move |opts: Dictionary| match opts.get("default_lib") {
        Some(dl) => {
            let dl_string = unsafe { dl.clone().into_string_unchecked().to_string() };
            let dl_path = PathBuf::from_str(&dl_string).unwrap();

            let mut sl = setup_lib.borrow_mut();
            *sl = Some(Library::open(&dl_path));
        }
        None => api::err_writeln("pinky error: no default lib enabled"),
    });

    let new_page_lib = library.clone();
    let new_page = Function::from_fn(move |opts: Dictionary| {
        let mut npl = new_page_lib.borrow_mut();
        if npl.is_none() {
            api::err_writeln("pinky error: no library loaded");
            return;
        }

        let schema = match opts.get("schema") {
            Some(s) => Some(unsafe { s.clone().into_string_unchecked().to_string() }),
            None => None,
        };

        let lib = npl.as_mut().unwrap();
        let page_path = lib.new_page(schema);
        let page_path_string = page_path.to_str().unwrap().to_string();

        api::command(&format!("e {}", page_path_string)).unwrap();
    });

    let pinky_nvim = Dictionary::from_iter([("setup", setup), ("new_page", new_page)]);
    Ok(pinky_nvim)
}
