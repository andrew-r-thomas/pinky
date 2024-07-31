use std::{
    collections::HashSet,
    fs::{read, read_dir, File, ReadDir},
    io::Write,
};

use clap::Parser as ClapParser;
use stringmetrics::levenshtein;
use tree_sitter::{Parser, Query, QueryCursor};

/*



*/

#[derive(ClapParser)]
struct CheckDups {
    path: std::path::PathBuf,

    #[arg(short, long)]
    thresh: usize,

    #[arg(short, long)]
    out: String,
}

fn main() {
    let args = CheckDups::parse();
    let mut parser = Parser::new();
    let language = tree_sitter_pinky::language();
    parser
        .set_language(&language)
        .expect("failed to load pinky grammar");

    let wiki_link_query = Query::new(&language, "(link_text) @link_text").unwrap();

    let read_dir = read_dir(args.path).expect("not a valid directory");
    let mut link_table = Vec::<HashSet<String>>::new();

    build_link_table(
        &mut link_table,
        read_dir,
        &wiki_link_query,
        &mut parser,
        args.thresh,
    );

    let mut out_file = File::create(args.out).unwrap();
    link_table.sort_by(|a, b| b.len().cmp(&a.len()));

    for dups in link_table {
        if dups.len() == 1 {
            break;
        }

        out_file.write_all(b"group:\n").unwrap();
        for d in dups {
            out_file.write_all(format!("{}\n", d).as_bytes()).unwrap();
        }
        out_file.write_all(b"\n").unwrap();
    }
}

fn build_link_table(
    table: &mut Vec<HashSet<String>>,
    dir: ReadDir,
    query: &Query,
    parser: &mut Parser,
    thresh: usize,
) {
    for entry in dir {
        if let Ok(e) = entry {
            let file_type = e.file_type().unwrap();
            if file_type.is_file() {
                if let Some(extension) = e.path().extension() {
                    if extension == "md" {
                        // TODO: streaming, this is super slow for large files
                        let bytes = read(e.path()).unwrap();
                        let tree = parser.parse(bytes.clone(), None).unwrap();
                        let mut query_cursor = QueryCursor::new();
                        let text_provider = bytes.clone();
                        let wiki_links = query_cursor.matches(
                            &query,
                            tree.root_node(),
                            text_provider.as_slice(),
                        );

                        for wiki_link in wiki_links {
                            for capture in wiki_link.captures {
                                let node = capture.node;
                                let text = node.utf8_text(&text_provider).unwrap();
                                let mut added = false;
                                for list in &mut *table {
                                    if list.contains(text) {
                                        added = true;
                                        break;
                                    }
                                    let close = {
                                        let mut out = true;
                                        for link in list.iter() {
                                            let dist = levenshtein(link, text);
                                            if dist > thresh as u32 {
                                                out = false
                                            }
                                        }
                                        out
                                    };
                                    if close {
                                        list.insert(text.to_string());
                                        added = true;
                                        break;
                                    }
                                }
                                if !added {
                                    let mut new = HashSet::new();
                                    new.insert(text.to_string());
                                    table.push(new);
                                }
                            }
                        }
                    }
                }
            } else if file_type.is_dir() {
                build_link_table(table, read_dir(e.path()).unwrap(), query, parser, thresh);
            }
        }
    }
}
