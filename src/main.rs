// #![feature(slice_group_by)]
extern crate termion;


use termion::screen::IntoAlternateScreen;
use std::io::{Write, stdout, stdin, Read};
use std::sync::{Arc, Mutex, Condvar};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;


fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let child = std::process::Command::new(&args[1])
                        .arg(args[2..].join(" "))
                        .stdout(std::process::Stdio::piped())
                        .spawn()
                        .expect("failed to execute child");
    
                    
    let mutex_slides = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));
    let mutex_slides_ = mutex_slides.clone();
    let finished = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    std::thread::spawn( move || {
        let mut unfinished_slide = Option::<Vec<String>>::None;
        let mut buffer = [0u8; 100000];
        let mut pipe = child.stdout.expect("no stdout for child");
        let mut not_started_ = true;
        while let Result::Ok(len) = pipe.read(&mut buffer) {
            if len == 0 {
                break;
            }
            let mut slides = mutex_slides_.lock().expect("cant unlock mutex");
            let mut split = buffer[..len].split(|c| c == &0u8).map(|s| {
                std::str::from_utf8(s).expect("std::str::from_utf8 failed").split_inclusive('\n').map(|s|{
                    let mut ret = s.to_string();
                    ret.push('\r');
                    ret.push_str(&termion::clear::CurrentLine.to_string());
                    ret 
                })    
                .collect::<Vec<String>>()
            })
            .filter(|s| s.len() != 0)
            .collect::<Vec<Vec<String>>>();

            if let Some(mut s) = unfinished_slide.take() {
                s.append(&mut split.remove(0));
                slides.push(s);
            }
            if buffer[len - 1] != 0x0  {
                unfinished_slide = split.pop()
            }
            
            slides.append(&mut split);
            
            if slides.len() >= 1 && not_started_ {
                let (lock, cvar) = &*pair2;
                let mut started = lock.lock().unwrap();            
                *started = true;
                not_started_ = false;
                // We notify the condvar that the value has changed.
                cvar.notify_one();
            }
        }
        finished.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    
    
    let mut index = 0usize;
    let mut screen = stdout().into_raw_mode().unwrap().into_alternate_screen().unwrap();
    write!(screen, "{}", termion::cursor::Hide).unwrap();
    let (_width, height) = termion::terminal_size().expect("cant get terminal size");
    

    {
        let slides = mutex_slides.lock().expect("can unlock mutex_slides");
        let start = match slides[index].len() > height.into()  {
            true => slides[index].len() + 2 - (height as usize),
            false => 0,

        };

        write!(screen, "{}{}{}slide: {} of {}\n\r", termion::cursor::Goto(1,1), termion::clear::CurrentLine, slides[index][start..].join(""), index+1, slides.len()).expect("write failed");
        screen.flush().expect("flush failed");
    }

    

    for c in stdin().keys() {
        let slides = mutex_slides.lock().expect("can unlock mutex_slides");
        match c.unwrap() {
            Key::Ctrl('c') => break,
            Key::Esc => break,
            Key::Char('q') => break,
            Key::Left => index = index.checked_sub(1).unwrap_or(index),
            Key::Right => index = std::cmp::min(index+1, slides.len()),
            Key::Up => index = index.checked_sub(1).unwrap_or(index),
            Key::Down => index = std::cmp::min(index+1, slides.len()),
            _ => {}
        }

        if index < slides.len() {
            let start = match slides[index].len() > height.into()  {
                true => slides[index].len() + 1 - (height as usize),
                false => 0,
    
            };
            write!(screen, "{}{}{}slide: {} of {}\n\r", termion::cursor::Goto(1,1), termion::clear::CurrentLine, slides[index][start..].join(""), index + 1, slides.len()).expect("write failed");
            //TODO: use size to clip
            //size was buggy on old WSL, and I forgot what I was doing
            //let size = termion::terminal_size().expect("cant get terminal size");
            //write!(screen, "start: {}, height {}, len: {}, size: {:?}", start, slides[index].len(), height, size).unwrap();
            screen.flush().expect("flush failed");
        }
        
    }
    write!(screen, "{}", termion::cursor::Show).unwrap();
    
}
