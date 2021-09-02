mod common;
mod convert_demo;
mod docx_xml;
mod libopencc;
mod xml_simp_parser;
mod ziphandler;
extern crate zip;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

//TODO: String &str in a mess

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Bad parameters.");
        return;
    }

    let docx_filename: &str = &args[1];
    let opencc_config_filename: &str = &args[2];

    // get accurate opencc config file path
    let exe_folder_path: &str = match Path::new(&args[0]).parent() {
        Some(x) => match x.to_str() {
            Some(y) => y,
            None => {
                println!("Invalid file path (exe).");
                return;
            }
        },
        None => {
            println!("Invalid parent path (exe).");
            return;
        }
    };
    let opencc_config_full_path: String = match Path::new(&opencc_config_filename).parent() {
        Some(x) => match x.to_str() {
            Some(y) => match y {
                "" => exe_folder_path.to_owned() + "/" + opencc_config_filename, // no parent folder, use exe folder
                _ => opencc_config_filename.to_owned(), // parent folder exists, use it
            },
            None => {
                println!("Invalid file path (opencc).");
                return;
            }
        },
        None => {
            println!("Invalid parent path (opencc).");
            return;
        }
    };
    println!(
        "[INFO] Using opencc config file at {}",
        opencc_config_full_path
    );
    if !Path::new(&opencc_config_full_path).is_file() {
        println!("[ERR] Opencc config file does not exits!");
        return;
    }

    // make preprations before conversion
    let docx_filename_new =
        &(docx_filename[0..docx_filename.len() - 5].to_owned() + "_modified.docx");
    let html_filename_new =
        &(docx_filename[0..docx_filename.len() - 5].to_owned() + "_forcheck.html");
    let docx_temp_dir: String = docx_filename.to_owned() + "____tmpdir_032932054"; // TODO: to make sure this dir does not exit
    ziphandler::extract(docx_filename, &docx_temp_dir);

    // convert
    let docx_sub_file_cnv_list = vec![
        "word/document.xml",
        "word/footnotes.xml",
        "word/endnotes.xml",
    ];
    let mut html_content: String = String::new();
    for docx_sub_file in docx_sub_file_cnv_list {
        println!("[Info] Deal with {}", docx_sub_file);
        let docx_sub_file_full_path = docx_temp_dir.to_owned() + "/" + docx_sub_file;
        if !Path::new(&docx_sub_file_full_path).exists() {
            println!("[Info] {} does not exit. Skip.", docx_sub_file);
            continue;
        }
        let xml_raw: String = match fs::read_to_string(&docx_sub_file_full_path) {
            Ok(x) => x,
            Err(err) => {
                println!("[ERR] {}", err);
                return;
            }
        };

        let mut xmlparser = xml_simp_parser::XMLSimpParser::new(&xml_raw);
        let mut docxobj = docx_xml::DocxXMl::new(&mut xmlparser, &opencc_config_full_path);
        let zi_sum_original: usize = docxobj.get_zi_sum();
        let mut xml_converted: String = String::new();
        let mut html_converted: String = String::new();
        let zi_sum_converted: usize = docxobj.convert(&mut xml_converted, &mut html_converted);
        println!(
            "Original: {}\tConverted: {}",
            zi_sum_original, zi_sum_converted
        );

        let mut file = match fs::File::create(&docx_sub_file_full_path) {
            Ok(x) => x,
            Err(err) => {
                println!("[ERROR] {}", err);
                return;
            }
        };
        match file.write_all(xml_converted.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                println!("[ERROR] {}", err);
                return;
            }
        }

        html_content = format!(
            "{}{}{}{}{}{}",
            html_content,
            "<div class=\"",
            docx_sub_file,
            "\">\n",
            html_converted,
            "</div>\n<br />\n"
        );
    }
    html_content =
        format! {"{}{}{}","<!DOCTYPE html>\n<html>\n<body>\n",html_content,"</body>\n</html>"};
    //duplicate code
    let mut file = match fs::File::create(html_filename_new) {
        Ok(x) => x,
        Err(err) => {
            println!("[ERROR] {}", err);
            return;
        }
    };
    match file.write_all(html_content.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            println!("[ERROR] {}", err);
            return;
        }
    }

    match ziphandler::do_zip_dir(
        &docx_temp_dir,
        docx_filename_new,
        zip::CompressionMethod::DEFLATE,
    ) {
        Ok(_) => {}
        Err(err) => {
            // TODO: err code macro?
            println!("[ERROR] {}", err);
            return;
        }
    }

    match fs::remove_dir_all(docx_temp_dir) {
        Ok(_) => {}
        Err(err) => {
            println!("[ERROR] {}", err);
            return;
        }
    };

    println!("Done.")
}
