use clap::{arg, Command};
use std::thread::sleep;
use std::time::Duration;
use scraper::{Html, Selector};


/*

The Unoffical Groove Stats Auto Pack Downloader
Copyright (C) 2022  dimo_jr <1s16slrse@mozmail.com>

This program is free software; you can redistribute it and/or
modify it under the terms of the GNU General Public License
as published by the Free Software Foundation; either version 2
of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.

*/
 
/*⚠️ Always set higher and never change below 100 or you may take down a resource server accidently. ⚠️*/   
static MINIMUM_TIMEOUT : u64 = 100; 

#[derive(Debug,Clone)]
struct CommandOptions {
    timeout_per_download : u64, // u64 to avoid casting and higher is better for the server that is hosting the song packs so olease be mindful
    #[allow(dead_code)]
    save_pack_path : String, // Location to save the path once the packs are extracted
}

impl CommandOptions {
    fn new(timeout_per_download : u64, save_pack_path : String) -> Self {
        let timeout = if timeout_per_download < MINIMUM_TIMEOUT { // Ensure we don't crash the server by respecting the server owner's precious resources
            MINIMUM_TIMEOUT
        }
        else {
            timeout_per_download
        };
        CommandOptions {
           timeout_per_download: timeout,
           save_pack_path : save_pack_path
        }
    }
}

impl Default for CommandOptions {
    fn default() -> Self {
        CommandOptions::new(MINIMUM_TIMEOUT, String::from(""))
    }
}

fn get_command_options() -> CommandOptions {
    let _matches = Command::new("Groovestats Pack Auto Downloader Command Line Tool")
    .version(env!("CARGO_PKG_VERSION"))
    .author("dimo_jr") 
    .about("Download all groove stat")
    .arg(arg!(
        -t --timeout <TIMEOUT> "Time out in milliseconds to avoid taking down StepmaniaOnline servers"
    ).required(false))
    .arg(arg!(
        -p --pack_download_path <PATH> ... "Where the GrooveStats packs extracted will be placed."
    ).required(false))
    .get_matches();
    CommandOptions::new(
        100,
        String::from(""),
    )
}



fn get_song_list(endpoint : &str) -> Vec<String> {
    if let Ok(response) = reqwest::blocking::get(endpoint) {
        let html_text = response.text().unwrap_or_default();
        let html = Html::parse_document(html_text.as_str());
        let selector = Selector::parse("option");
        let elements = html.select(selector.as_ref().unwrap());
        elements.skip(6).map(|e| e.inner_html()).collect()
    }
    else {
        vec![]
    }
}

fn download_song_pack(_song_pack_name : &String) {
    
}

fn main() {
   let args = get_command_options();
   let groovestat_https_endpoint = "https://groovestats.com/index.php?page=songscores&gameid=1";
   println!("Recieving song pack list from groove stats from the following endpoint: {}", groovestat_https_endpoint);
   for song_pack_name in &get_song_list(groovestat_https_endpoint) {
       if song_pack_name.is_empty() { // In case the web scraping fails
           continue;
       }
       println!("Downloading {} ...", song_pack_name);
       download_song_pack(song_pack_name);
       sleep(Duration::from_millis(args.timeout_per_download));
   }
   println!("Finished downloading and extracting all packs");
   
}
