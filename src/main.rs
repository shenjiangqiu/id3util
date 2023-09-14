use clap::ColorChoice;
use clap::{Args, Command, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Generator, Shell};
use id3::Tag as Id3Tag;
use id3::TagLike;
use mp4ameta::Tag as Mp4Tag;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    sync::atomic::AtomicUsize,
};

#[derive(Parser, Debug)]
#[command(author("Jiangqiu Shen"), version,about,long_about("id3 tag tools"),color=ColorChoice::Auto, disable_colored_help=false)]
struct CmdArgs {
    #[command(subcommand)]
    subcmd: Commands,
}

#[derive(Debug, Clone, ValueEnum)]
enum ExtType {
    Mp3,
    M4a,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// list all tags with ext
    List { ext: ExtType, name: PathBuf },
    /// write empty tags
    Write { name: PathBuf },
    /// write smart tags according to file name
    WriteSmart {
        path: PathBuf,
        author: String,
        album: String,
    },
    /// convert acc to mp3
    Convert {
        path: PathBuf,
        old: String,
        new: String,
    },
    /// rename the files
    Number { path: PathBuf, ext: ExtType },
    /// set a single file
    Set(SetArgs),
    /// generate completion script
    Generate { shell: Shell },
}
#[derive(Args, Debug)]
struct SetArgs {
    /// the artist
    #[arg(short, long)]
    artist: Option<String>,
    /// the album
    #[arg(short('l'), long)]
    album: Option<String>,
    /// the title
    #[arg(short, long)]
    title: Option<String>,
    /// the track number
    #[arg(short('r'), long)]
    track: Option<u16>,
    /// the filename
    file_name: PathBuf,
}

trait CommonTag {
    fn ext_name() -> &'static str;
    fn read_from_path(path: &Path) -> Self;

    fn artist(&self) -> Option<&str>;
    fn album(&self) -> Option<&str>;
    fn title(&self) -> Option<&str>;
    fn album_artist(&self) -> Option<&str>;

    fn set_artist(&mut self, name: &str);
    fn set_album_artist(&mut self, name: &str);
    fn set_album(&mut self, name: &str);
    fn set_title(&mut self, name: &str);
    fn set_track(&mut self, d: u16, t: u16);
    fn set_disc(&mut self, d: u16, t: u16);
    fn write_to_path(&self, name: &str);
}
impl CommonTag for Mp4Tag {
    fn read_from_path(path: &Path) -> Self {
        let tag = Mp4Tag::read_from_path(path).unwrap();
        tag
    }

    fn set_artist(&mut self, name: &str) {
        self.set_artist(name);
    }
    fn set_album_artist(&mut self, name: &str) {
        self.set_album_artist(name);
    }
    fn set_album(&mut self, name: &str) {
        self.set_album(name);
    }
    fn set_title(&mut self, name: &str) {
        self.set_title(name);
    }
    fn set_track(&mut self, d: u16, t: u16) {
        self.set_track(d, t);
    }
    fn set_disc(&mut self, d: u16, t: u16) {
        self.set_disc(d, t);
    }
    fn write_to_path(&self, name: &str) {
        self.write_to_path(name).unwrap();
    }

    fn artist(&self) -> Option<&str> {
        self.artist()
    }

    fn album(&self) -> Option<&str> {
        self.album()
    }

    fn title(&self) -> Option<&str> {
        self.title()
    }

    fn album_artist(&self) -> Option<&str> {
        self.album_artist()
    }

    fn ext_name() -> &'static str {
        "m4a"
    }
}
impl CommonTag for Id3Tag {
    fn read_from_path(path: &Path) -> Self {
        let tag = Id3Tag::read_from_path(path).unwrap();
        tag
    }
    fn set_artist(&mut self, name: &str) {
        TagLike::set_artist(self, name);
    }
    fn set_album_artist(&mut self, name: &str) {
        TagLike::set_album_artist(self, name);
    }
    fn set_album(&mut self, name: &str) {
        TagLike::set_album(self, name);
    }
    fn set_title(&mut self, name: &str) {
        TagLike::set_title(self, name);
    }
    fn set_track(&mut self, d: u16, t: u16) {
        TagLike::set_track(self, d as u32);
        TagLike::set_total_tracks(self, t as u32);
    }
    fn set_disc(&mut self, d: u16, t: u16) {
        TagLike::set_disc(self, d as u32);
        TagLike::set_total_discs(self, t as u32);
    }
    fn write_to_path(&self, name: &str) {
        self.write_to_path(name, id3::Version::Id3v24).unwrap();
    }

    fn artist(&self) -> Option<&str> {
        TagLike::artist(self)
    }

    fn album(&self) -> Option<&str> {
        TagLike::album(self)
    }

    fn title(&self) -> Option<&str> {
        TagLike::title(self)
    }

    fn album_artist(&self) -> Option<&str> {
        TagLike::album_artist(self)
    }

    fn ext_name() -> &'static str {
        "mp3"
    }
}

fn main() {
    let args: CmdArgs = CmdArgs::parse();
    match args.subcmd {
        Commands::List { ext, name } => match ext {
            ExtType::Mp3 => {
                let id3s: Vec<Id3Tag> = get_id3(&name);
                for id3 in id3s {
                    println!(
                        "art: {:?} alb_art: {:?} alb: {:?} track: {:?} disc: {:?}",
                        CommonTag::artist(&id3),
                        CommonTag::album_artist(&id3),
                        CommonTag::album(&id3),
                        id3.track(),
                        id3.disc()
                    );
                }
            }
            ExtType::M4a => {
                let mp4s: Vec<Mp4Tag> = get_id3(&name);
                for mp4 in mp4s {
                    println!(
                        "art: {:?} alb_art: {:?} alb: {:?} track: {:?} disc: {:?}",
                        CommonTag::artist(&mp4),
                        CommonTag::album_artist(&mp4),
                        CommonTag::album(&mp4),
                        mp4.track(),
                        mp4.disc()
                    );
                }
            }
        },
        Commands::Write { name: _ } => {
            // write_empty_id3(&name);
            todo!()
        }
        Commands::WriteSmart {
            path,
            author,
            album,
        } => {
            write_smart_id3(&path, &author, &album);
        }
        Commands::Convert { path, old, new } => {
            if old == "aac" {
                if new == "mp3" || new == "m4a" {
                    convert_to_new_format(&path, &old, &new);
                } else {
                    panic!("Unknown extension")
                }
            } else {
                panic!("Unknown extension")
            }
        }
        Commands::Set(SetArgs {
            artist,
            album,
            title,
            file_name,
            track,
        }) => {
            let ext = file_name.extension().unwrap();
            if ext == "mp3" {
                let mut id3 = Id3Tag::read_from_path(&file_name).unwrap_or_default();
                if let Some(artist) = artist {
                    CommonTag::set_artist(&mut id3, &artist);
                    CommonTag::set_album_artist(&mut id3, &artist);
                }
                if let Some(album) = album {
                    CommonTag::set_album(&mut id3, &album);
                }
                if let Some(title) = title {
                    CommonTag::set_title(&mut id3, &title);
                }
                if let Some(track) = track {
                    CommonTag::set_track(&mut id3, track, 1);
                }
                CommonTag::write_to_path(&id3, &file_name.to_str().unwrap());
            } else if ext == "m4a" {
                let mut mp4 = Mp4Tag::read_from_path(&file_name).unwrap_or_default();
                if let Some(artist) = artist {
                    mp4.set_artist(artist);
                }
                if let Some(album) = album {
                    mp4.set_album(album);
                }
                if let Some(title) = title {
                    mp4.set_title(title);
                }
                if let Some(track) = track {
                    mp4.set_track(track, 1);
                }
                mp4.write_to_path(&file_name).unwrap();
            } else {
                panic!("Unknown extension")
            }
        }
        Commands::Number { path, ext } => {
            let ext_string = match ext {
                ExtType::Mp3 => "mp3",
                ExtType::M4a => "m4a",
            };
            let mut entry_names = std::fs::read_dir(path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|entry| {
                    entry
                        .extension()
                        .map(|ext| ext == ext_string)
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            // sort the path
            entry_names.sort_unstable();
            // rename the files prefix a number from 1 to n
            for (i, entry) in entry_names.iter().enumerate() {
                let new_name =
                    format!("{}-{}", i + 1, entry.file_name().unwrap().to_str().unwrap());
                let new_path = entry.with_file_name(new_name);
                std::fs::rename(entry, new_path).unwrap();
            }
        }
        Commands::Generate { shell } => {
            let mut command = CmdArgs::command();
            print_completions(shell, &mut command);
        }
    }
}
fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
fn get_id3<T: CommonTag, P: AsRef<Path>>(folder_name: P) -> Vec<T> {
    let mut tags = Vec::new();
    for entry in std::fs::read_dir(folder_name).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == T::ext_name() {
            let tag = T::read_from_path(&path);

            tags.push(tag);
        }
    }
    tags
}

fn write_smart_id3(folder_name: impl AsRef<Path>, author: &str, album: &str) {
    println!(
        "Writing smart id3 for {:?} {author} {album}",
        folder_name.as_ref()
    );
    let entry_names = std::fs::read_dir(folder_name)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();

    let entry_names_with_audio_extension = entry_names
        .iter()
        .filter(|entry| {
            entry.extension() == Some(OsStr::new("mp3"))
                || entry.extension() == Some(OsStr::new("m4a"))
        })
        .collect::<Vec<_>>();
    // build a regex to find the first number in the file name
    let re = regex::Regex::new(r"(\d+)").unwrap();
    let track_nums = entry_names_with_audio_extension
        .iter()
        .map(|&entry| {
            let file_name = entry.file_name().unwrap().to_str().unwrap();
            let caps = re.captures(file_name).unwrap();
            let track_num = caps.get(1).unwrap().as_str().parse::<u16>().unwrap();
            (entry, track_num)
        })
        .collect::<Vec<_>>();
    println!("Track nums: {:?}", track_nums);
    // prompt to confirm
    println!("Confirm? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(32)
        .build_global()
        .unwrap();
    let total_files = AtomicUsize::new(track_nums.len());
    if input.trim() == "y" {
        track_nums.par_iter().for_each(|(entry, track_num)| {
            let ext_name = entry.extension().unwrap().to_str().unwrap();
            let ext_type = match ext_name {
                "mp3" => ExtType::Mp3,
                "m4a" => ExtType::M4a,
                _ => panic!("not an audio file"),
            };
            match ext_type {
                ExtType::Mp3 => {
                    let mut tag = Id3Tag::read_from_path(entry).unwrap_or_default();
                    write_tag(
                        &mut tag,
                        author,
                        album,
                        entry.file_name().unwrap().to_str().unwrap(),
                        entry.to_str().unwrap(),
                        *track_num,
                        track_nums.len() as u16,
                    )
                }
                ExtType::M4a => {
                    let mut tag = Mp4Tag::read_from_path(entry).unwrap_or_default();
                    write_tag(
                        &mut tag,
                        author,
                        album,
                        entry.file_name().unwrap().to_str().unwrap(),
                        entry.to_str().unwrap(),
                        *track_num,
                        track_nums.len() as u16,
                    )
                }
            };
            let now = total_files.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            println!("{} files left", now);
        });
    }
}

fn write_tag<T: CommonTag>(
    tag: &mut T,
    author: &str,
    album: &str,
    name: &str,
    path: &str,
    track: u16,
    total_tracks: u16,
) {
    tag.set_artist(author);
    println!("Setting artist to {:?}", author);
    tag.set_album_artist(author);
    println!("Setting album artist to {:?}", author);
    tag.set_album(album);
    tag.set_title(&format!("Track-{}-{}", track, name));
    tag.set_track(track, total_tracks);
    tag.set_disc(1, 1);
    tag.write_to_path(path);
    println!("Wrote tag for {:?}", path);
}

fn convert_to_new_format(path: impl AsRef<Path>, old: impl AsRef<OsStr>, new: impl AsRef<OsStr>) {
    let entry_names = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    // filter the files with the old extension
    let entry_names_with_old_extension = entry_names
        .iter()
        .filter(|entry| entry.extension() == Some(old.as_ref()))
        .collect::<Vec<_>>();
    // use ffmpeg to convert the files
    // ffmpeg -i old_file -codec: copy new_file
    let mut remaining = entry_names_with_old_extension.len();
    for entry in entry_names_with_old_extension {
        let new_file_name = entry.with_extension(new.as_ref());
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-i")
            .arg(entry)
            .arg("-codec:")
            .arg("copy")
            .arg(new_file_name);
        println!("Running {:?}", cmd);
        let output = cmd.output().unwrap();
        println!("Output: {:?}", output);
        remaining -= 1;
        println!("{} files left", remaining);
    }
}
