use clap::{arg, Command};
use colored::Colorize;
use scraper::{Html, Selector};
#[allow(unused_imports)] //TODO: Remove once used
use std::fs::File;
#[allow(unused_imports)] //TODO: Remove once used
use std::io::copy;
use std::thread::sleep;
use std::time::Duration;

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
static MINIMUM_TIMEOUT: u64 = 100; // Used as a delay not to be confused with HTTP timeout although this constant maybe renamed to avoid confusion with the term used by TCP sockets

// To avoid having to type the tuple everywhere -- Rust has a very strict type system
type RGB = (u8, u8, u8);

//Colors (in RGB not Hex) that will be used by this tool feel free to change the values here as the names will be color neutral
static ERROR_RGB: RGB = (255, 0, 0);
static OK_RGB: RGB = (245, 66, 155);
static FINISHED_RGB: RGB = (0, 255, 0);

fn debug() -> bool {
    cfg!(debug_assertions)
}

//-----------------------------------------------------------------------------------------------------------------

//FIXME: Eventually move these functions into their own source file

//FIXME: Refactor to a macro to improve performance if necessary
#[allow(dead_code)] //FIXME: Remove once used in the near future or depecrated if it will never be used and eventually removed in a future commit
fn colored_print(string: String, rgb: RGB) {
    print!("{}", string.truecolor(rgb.0, rgb.1, rgb.2));
}
//FIXME: Refactor this as well to a macro in the future for the same reasons as above
fn colored_println(string: String, rgb: RGB) {
    println!("{}", string.truecolor(rgb.0, rgb.1, rgb.2));
}

fn debug_println(string: &String) {
    if debug() {
        colored_println(format!("DEBUG: {}", string), FINISHED_RGB);
    }
}

#[allow(dead_code)] //FIXME: Remove once used in the near future or depecrated if it will never be used and eventually removed in a future commit
fn debug_print(string: &String) {
    if debug() {
        colored_print(format!("DEBUG: {}", string), FINISHED_RGB);
    }
}

//-----------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct CommandOptions {
    timeout_per_download: u64, // u64 to avoid casting and higher is better for the server that is hosting the song packs so olease be mindful
    save_pack_path: String, // Location to save the path once the packs are extracted
}

impl CommandOptions {
    fn new(timeout_per_download: u64, save_pack_path: String) -> Self {
        let timeout = if timeout_per_download < MINIMUM_TIMEOUT {
            // Ensure we don't crash the server by respecting the server owner's precious resources
            MINIMUM_TIMEOUT
        } else {
            timeout_per_download
        };
        CommandOptions {
            timeout_per_download: timeout,
            save_pack_path: save_pack_path,
        }
    }

    fn print(&self) {
        colored_println(format!("CONFIGURATION: Thread Sleep: {} [ms] Save To: [{}] ", self.timeout_per_download, self.save_pack_path), FINISHED_RGB);
    }
}

impl Default for CommandOptions {
    fn default() -> Self {
        CommandOptions::new(MINIMUM_TIMEOUT, String::from(""))
    }
}

fn stepmania_default_path() -> String {
    // ⚠️ This assumes stepmania -- if you use a fork or derivative of stepmania feel free to change the defaults per platform
    if cfg!(windows) {
        //Default assumes current Stepmania version as of writing will update to default with the latest Stepmania major release version
        String::from("C:\\Games\\StepMania 5\\Songs") // TODO: Replace with Stepmania default song pack path for Windows systems
    } else if cfg!(unix) {
        String::default() // TODO: Replace with Stepmania default song pack path for Linux systems
    } else {
        String::from(std::env::current_dir().unwrap_or_default().to_str().unwrap_or_default()) // Only executes if the platform operating system is uknown. We return empty string, which means packs will be saved in current working directory; refer to cd or equivalent command to change the path this should be avoided since most people don't use TempleOS for example.
    }
}

fn get_command_options() -> CommandOptions {
    let _matches = Command::new("Groovestats Pack Auto Downloader Command Line Tool")
    .version(env!("CARGO_PKG_VERSION"))
    .author("dimo_jr <1s16slrse@mozmail.com>") 
    .about("Download all groovestats pack respectfully and securely please donate or help out to groovestats. ")
    .arg(arg!(
        -t --timeout <TIMEOUT> "Time out in milliseconds to avoid taking down StepmaniaOnline servers"
    ).required(false))
    .arg(arg!(
        -p --pack_download_path <PATH> ... "Where the GrooveStats packs extracted will be placed."
    ).required(false))
    .get_matches();
    CommandOptions::new(MINIMUM_TIMEOUT, stepmania_default_path())
}

fn get_song_list(endpoint: &str) -> Vec<String> {
    if let Ok(response) = reqwest::blocking::get(endpoint) {
        let html_text = response.text().unwrap_or_default();
        let html = Html::parse_document(html_text.as_str());
        let selector = Selector::parse("option");
        let elements = html.select(selector.as_ref().unwrap());
        elements
            .skip(52)
            .map(|e| String::from(e.inner_html().trim()))
            .collect()
    } else {
        vec![]
    }
}

//Called when the downloaded pack resource has invalid bytes to avoid extracting a corrupted song pack
fn unknown_zip_blob_error() {
    colored_println(format!("{}","Song pack repo was unable to extract zip blob please check network on your end or wait for repo to have a more stable connection."), ERROR_RGB);
}

fn download_song_pack(song_pack_name: &String, args: &CommandOptions) {
    //FIXME: Move these to global namespace if it will be needed elsewhere in a future revision
    use urlencoding::encode; 
    let download_url = &format!(
        "https://search.stepmaniaonline.net/static/new/{}.zip", //FIXME: Potential problem is that stepmania online may not host every pack that is part of GrooveStats eventually I will refactor to take a json config file that lists all a possible mirrors like Zenius for example.
        encode(song_pack_name)
    );
    debug_println(download_url);
    if let Ok(response) = reqwest::blocking::get(download_url) {
        colored_println(
            format!("Extracting '{}.zip' into system memory", song_pack_name),
            OK_RGB,
        );
        if let Ok(zip_blob) = response.bytes() {
            if zip_blob.len() > 0 {
                let compressed_size = zip_blob.len();
                use std::io::Cursor; //FIXME: Move it to global namespace if it will be needed elsewhere in a future revision
                let ram_file = Cursor::new(zip_blob);
                if let Ok(archive) = zip::ZipArchive::new(ram_file).as_mut() {
                    let pack_directory_path = format!("{}{}",args.save_pack_path, std::path::MAIN_SEPARATOR);
                    if archive.extract(&pack_directory_path).is_err() {
                        unknown_zip_blob_error();
                    }
                    else {
                        use std::path::Path;
                        use filesize::PathExt;
                        let uncompressed_size = Path::new(&pack_directory_path).size_on_disk().unwrap_or_default() as usize;
                        colored_println(
                            format!(
                                "Song Pack: {} [{}] Compressed [RAM] (Bytes): {} Uncompressed [Disk] (Bytes): {} Extracted: [✔️]", //FIXME: Check if Terminal supports Unicode and provide a fallback in case the Terminal is from the stone ages
                                song_pack_name, pack_directory_path, compressed_size, uncompressed_size
                            ),
                            FINISHED_RGB,
                        );
                    }
                }
                else {  
                    unknown_zip_blob_error();
                }
            } else {
                unknown_zip_blob_error();
            }
        } else {
            unknown_zip_blob_error();
        }
        colored_println(
            format!(
                "Saving pack on disk on {}{}{}",
                args.save_pack_path,
                std::path::MAIN_SEPARATOR,
                song_pack_name
            ),
            OK_RGB,
        );
    } else {
        colored_println(
            format!("Unable to download at '{}' skipping ...", download_url),
            ERROR_RGB,
        );
    }
}

fn main() {
    let args = get_command_options();
    args.print();
    let groovestat_https_endpoint = "https://groovestats.com/index.php?page=songscores&gameid=1";
    colored_println(
        format!(
            "Recieving song pack list from groove stats from the following endpoint: {}",
            groovestat_https_endpoint
        ),
        OK_RGB,
    );
    for song_pack_name in &get_song_list(groovestat_https_endpoint) {
        if song_pack_name.is_empty() {
            // In case the web scraping fails
            continue;
        }
        format!("Downloading {} ...", song_pack_name);
        download_song_pack(song_pack_name, &args);
        sleep(Duration::from_millis(args.timeout_per_download));
    }
    colored_println(
        String::from("Finished downloading and extracting all packs"),
        FINISHED_RGB,
    );
}
