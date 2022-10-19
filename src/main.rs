use std::{env,process,str};
use std::path::PathBuf;
use std::io::{stdout,Write};
use std::fs::File;
use futures_util::StreamExt;
use serde_json;
use reqwest::Client;
use colored::*;
use enable_cmd_ansi_colors;

struct ImageInfo {
    filename: String,
    tim: String,
    ext: String
}

async fn download(path: PathBuf,board: &str,images: Vec<ImageInfo>){
    let req_client = Client::new();
    let post_count = images.len();
    for (index,image) in images.iter().enumerate() {
        let filepath = path.join(format!("{}.{}",image.filename,image.ext));
        if filepath.exists() {
            println!("[{}][{}][Existente]{}",format!("{}/{}",index+1,post_count),"100%".green(),format!("{}{}",image.filename,image.ext).green());
            continue;
        }
        let con = req_client.get(format!("https://i.4cdn.org/{}/{}{}",board,image.tim,image.ext)).send().await.unwrap();
        let size: f64 = con.content_length().unwrap() as f64;
        let mut done: f64 = 0.0;

        let mut file = File::create(&filepath).unwrap();
        let mut stream = con.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = match item {
                Ok(item) => item,
                Err(_) => {
                    println!("Erro ao baixar: {}.{}",image.filename,image.ext);
                    continue;
                }
            };
            file.write_all(&chunk).expect(&format!("Erro ao escrever aquivo: {}",&filepath.to_string_lossy()));
            done += chunk.len() as f64;
            let perc = ((done/size)*100.0) as i32;
            print!("\r[{}][{}][{}]{}",format!("{}/{}",index+1,post_count),
                if perc == 100 {format!("{}%",perc).green()} else {format!("{}%",perc).yellow()},
                format!("{:.2}mb",size/(1024.0*1024.0)),
                if perc == 100 {format!("{}{}",image.filename,image.ext).green()} else {format!("{}{}",image.filename,image.ext).yellow()}
            );
            stdout().flush().unwrap();
        }
        println!();
    }
}

async fn get_all_images(board: &str,thread: &str) -> Vec<ImageInfo> {
    let resp = reqwest::get(format!("https://a.4cdn.org/{}/thread/{}.json",board,thread))
    .await.unwrap()
    .text()
    .await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let mut i_data = Vec::new();
    for i in json["posts"].as_array().unwrap() {
        let obj = i.as_object().unwrap();
        if obj.contains_key("tim") {
            i_data.push(
                ImageInfo { 
                    filename: obj["filename"].as_str().unwrap().to_owned(), 
                    tim: obj["tim"].as_i64().unwrap().to_string(), 
                    ext: obj["ext"].as_str().unwrap().to_owned() 
                }
            )
        }
    }
    i_data
}

#[tokio::main]
async fn main(){
    enable_cmd_ansi_colors::enable().unwrap();
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Uso 4chan_scraper <URL> <PATH>");
        return;
    }
    if !args[1].contains("boards.4channel.org") & !args[1].contains("boards.4chan.org") {
        println!("Link inválido.");
        process::exit(1)
    }
    let url = args[1].split("/").collect::<Vec<&str>>();
    let board = url[3];
    let thread = url[5];
    let path = PathBuf::from(&args[2]);
    let i_data = get_all_images(board, thread).await;
    download(path, board, i_data).await;
    println!("Todos os aquivos foram baixados !")
}