//pub fn annex_add(list: &[PathBuf]) -> Result<Vec<(Key, String)>> {
//    let mut child = Command::new("git")
//        .args(&["annex", "add", "--json", "--json-error-messages", "--batch"])
//        .args(&["+RTS", "-N2"])
//        .stdin(Stdio::piped())
//        .stdout(Stdio::piped())
//        .spawn().unwrap();
//
//    let mut stdin = child.stdin.take()
//        .ok_or(io::Error::new(io::ErrorKind::UnexpectedEof, "child stdin was closed"))?;
//    let stdout = child.stdout.take()
//        .ok_or(io::Error::new(io::ErrorKind::UnexpectedEof, "child stdout was closed"))?;
//
//    let len = list.len();
//    let tp = thread::spawn(move || {
//        let mut out = Vec::with_capacity(len);
//
//        for line in BufReader::new(stdout).lines() {
//            let line = line?;
//            if !line.is_empty() {
//                let j = json::parse(&line)?;
//                out.push(j);
//            }
//        }
//
//        info!("Read all from annex");
//
//        Ok(out.iter_mut().map(|j: &mut json::JsonValue| {
//            let ks: String = j.remove("key").take_string().unwrap();
//            let k = Key::try_parse(&ks).unwrap();
//            let p: String = j.remove("file").take_string().unwrap();
//            let f: String = std::path::Path::new(&p).file_name().and_then(|p: &std::ffi::OsStr| p.to_str().map(str::to_string)).unwrap();
//            (k,f)
//        }).collect())
//    });
//
//    for f in list.iter() {
//        stdin.write_all(f.to_str().unwrap().as_bytes())?;
//        stdin.write("\n".as_bytes())?;
//        stdin.flush()?;
//    }
//
//    stdin.flush()?;
//    std::mem::drop(stdin);
//
//    child.wait()?;
//
//    tp.join().unwrap()
//}

use std::process::{
    Command,
    Stdio,
};
use std::io;

use futures::prelude::*;
use futures::io::{
    AsyncBufReadExt,
    BufReader,
    AllowStdIo,
};

use json;

use crate::annex::Annex;

type Key = String;
type File = String;
type AddResult = std::result::Result<(Key, File), String>;

/// Add files to annex. Equivalent to `git-annex add`
///
/// See `add_opt`
pub fn add(files: impl Stream<Item=String>) 
    -> (impl Future<Output=Result<(), io::Error>>, Result<impl Stream<Item=AddResult>, std::io::Error>)
{
    add_opt(files, false, false, false)
}

/// Add files to annex. Equivalent to `git-annex add`
///
/// This function returns two values: A Future for forwarding the file list into `git-annex`
/// and a Stream of return values from `git-annex` about annexed files.
///
/// **IMPORTANT**: You need to poll *both* at the *same* time. Given a large enough list of
/// files that stdin/stdout buffer the Stream *WILL NEVER* complete unless the Future is polled and
/// the Future *WILL NEVER* resolve unless the Stream is polled! If you are not using a reactor
/// or other event loop either poll them in separate threads or poll the future, then read the
/// Stream until it returns `Poll::Pending`, then rinse and repeat.
// TODO: Figure out lifetimes and then make that a Stream of &Path instead of String
pub fn add_opt(files: impl Stream<Item=String>, include_dotfiles: bool, force: bool, update: bool) 
    -> (impl Future<Output=Result<(), io::Error>>, Result<impl Stream<Item=AddResult>, std::io::Error>)
{
    let mut args = Vec::new();
    if include_dotfiles {
        args.push("--include-dotfiles");
    }
    if force {
        args.push("--force")
    }
    if update {
        args.push("--update")
    }

    let mut cmd = Command::new("git-annex")
        .args(&["add", "--json", "--json-error-messages", "--batch"])
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    let stdin = cmd.stdin.take().unwrap();
    let stdout = cmd.stdout.take().unwrap();

    // Make stdin/-out AsyncRead
    let stdin_a = AllowStdIo::new(stdin);
    let stdout_a = AllowStdIo::new(stdout);

    let stdin_b = stdin_a.into_sink();
    // FIXME: Proper error handling
    let f = files
        .map(|p| Ok(p))
        .forward(stdin_b);

    let stdout_b = BufReader::new(stdout_a);
    let stdout_l = stdout_b.lines();

    // FIXME: Properly handle the errors
    (f, Ok(stdout_l
        .filter_map(|l| futures::future::ready(l.ok()))
        .map(|l| {
            let mut j = json::parse(&l).unwrap();

            let k: String = j.remove("key").take_string().unwrap();
            let f: String = j.remove("file").take_string().unwrap();
            Ok((k,f))
        })))
}
