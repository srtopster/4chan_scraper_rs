use std::{env,process,str};
use std::path::Path;
use std::io::{stdout,Write};
use std::fs::File;
use futures_util::StreamExt;
use std::collections::HashMap;
use serde_json;
use reqwest;
use colored::*;

async fn download(path: &str,board: &str,tim: &str,filename: &str,ext: &str,post_count: &str){
    let filepath = Path::new(path).join(format!("{}{}",filename,ext));
    if filepath.exists(){
        println!("[{}][{}][Existente]{}",post_count,"100%".green(),format!("{}{}",filename,ext).green());
        return
    }
    let resp = reqwest::get(format!("https://i.4cdn.org/{}/{}{}",board,tim,ext)).await.unwrap();
    let size = resp.content_length().unwrap() as f64;
    let mut stream = resp.bytes_stream();

    let mut file = File::create(filepath).unwrap();
    let mut done: f64 = 0.0;
    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Erro ao baixar"))).unwrap();
        file.write_all(&chunk).unwrap();
        done += chunk.len() as f64;
        let perc = ((done/size)*100.0) as i32;
        print!("\r[{}][{}][{}]{}",post_count,if perc == 100 {format!("{}%",perc).green()} else {format!("{}%",perc).yellow()},format!("{:.2}mb",size/(1024.0*1024.0)),if perc == 100 {format!("{}{}",filename,ext).green()}else {format!("{}{}",filename,ext).yellow()});
        stdout().flush().unwrap();
    }
    println!("");
}

async fn get_all_images(board: &str,thread: &str) -> Vec<std::collections::HashMap<&'static str, String>> {
    let resp = reqwest::get(format!("https://a.4cdn.org/{}/thread/{}.json",board,thread))
    .await.unwrap()
    .text()
    .await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let mut data = Vec::new();
    for i in json["posts"].as_array().unwrap() {
        let obj = i.as_object().unwrap();
        if obj.contains_key("tim") {
            //println!("Filname: {}, Tim: {}, Ext: {}",obj["filename"].to_string().replace('"',""),obj["tim"].to_string().replace('"',""),obj["ext"].to_string().replace('"',""));
            data.push(HashMap::from([("filename",obj["filename"].to_string().replace('"',"")),("tim",obj["tim"].to_string().replace('"',"")),("ext",obj["ext"].to_string().replace('"',""))]));
        }
    }
    data
}

#[tokio::main]
async fn main(){
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Uso 4chan_scraper <URL> <PATH>");
        process::exit(1)
    }
    if !args[1].contains("boards.4channel.org") & !args[1].contains("boards.4chan.org") {
        println!("Link inválido.");
        process::exit(1)
    }
    let url = args[1].split("/").collect::<Vec<&str>>();
    let board = url[3];
    let thread = url[5];
    let path = &args[2];
    let i_data = get_all_images(board, thread).await;
    for (count,file) in i_data.iter().enumerate(){
        download(path, board, file["tim"].as_str(), file["filename"].as_str(), file["ext"].as_str(),format!("{}/{}",count+1,i_data.len()).as_str()).await;
    }
    println!("Todos os aquivos foram baixados !")
}