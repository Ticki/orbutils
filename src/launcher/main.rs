extern crate orbclient;

use std::env;
use std::fs::{self, File};
use std::process::Command;
use std::thread;

use orbclient::{BmpFile, Color, EventOption, Window};

use package::Package;

pub mod package;

fn get_packages() -> Vec<Package> {
    let mut packages: Vec<Package> = Vec::new();

    for entry_result in fs::read_dir("/apps/").unwrap() {
        let entry = entry_result.unwrap();
        if entry.file_type().unwrap().is_dir() {
            packages.push(Package::from_path(&("/apps/".to_string() + entry.file_name().to_str().unwrap())));
        }
    }

    packages
}

fn draw(window: &mut Window, packages: &Vec<Package>, shutdown: &BmpFile, selected: i32){
    let w = window.width();
    let h = window.height();
    window.set(Color::rgb(128, 128, 128));

    let mut x = 0;
    let mut i = 0;
    for package in packages.iter() {
        if package.icon.has_data() {
            let y = h as isize - package.icon.height() as isize;

            if i == selected {
                window.rect(x as i32, y as i32,
                                  package.icon.width() as u32, package.icon.height() as u32,
                                  Color::rgb(224, 224, 224));
            }

            window.image(x as i32, y as i32,
                        package.icon.width() as u32,
                        package.icon.height() as u32,
                        &package.icon);
            x = x + package.icon.width() as i32;
            i += 1;
        }
    }

    if shutdown.has_data() {
        x = w as i32 - shutdown.width() as i32;
        let y = h as isize - shutdown.height() as isize;

        if i == selected {
            window.rect(x as i32, y as i32,
                              shutdown.width() as u32, shutdown.height() as u32,
                              Color::rgb(224, 224, 224));
        }

        window.image(x as i32, y as i32,
                        shutdown.width() as u32, shutdown.height() as u32,
                        &shutdown);
        x = x + shutdown.width() as i32;
        i += 1;
    }

    window.sync();
}

fn draw_chooser(window: &mut Window, packages: &Vec<Package>, mouse_x: i32, mouse_y: i32){
    let w = window.width();

    window.set(Color::rgb(255, 255, 255));

    let mut y = 0;
    for package in packages.iter() {
        if mouse_y >= y as i32 && mouse_y < y + 32 {
            window.rect(0, y, w, 32, Color::rgb(128, 128, 128));
        }

        if package.icon.has_data() {
            window.image(0, y, package.icon.width() as u32, package.icon.height() as u32, &package.icon);
        }

        let mut c_x = 40;
        for c in package.name.chars() {
            window.char(c_x as i32, y + 8, c, Color::rgb(0, 0, 0));
            c_x += 8;
        }

        y += 32;
    }

    window.sync();
}

fn main() {
    let paths = env::args().skip(1);
    if paths.len() > 0 {
        for ref path in paths {
            let mut packages = get_packages();

            packages.retain(|package| -> bool {
                for accept in package.accepts.iter() {
                    if (accept.starts_with('*') && path.ends_with(&accept[1 ..])) ||
                       (accept.ends_with('*') && path.starts_with(&accept[.. accept.len() - 1])) {
                        return true;
                    }
                }
                false
            });

            if packages.len() > 1 {
                for package in packages.iter() {
                    println!("{:?}: {}", package.binary, package.icon.has_data());
                }

                let mut window = Window::new(-1, -1, 400, packages.len() as u32 * 32, path).unwrap();

                draw_chooser(&mut window, &packages, -1, -1);
                'choosing: loop {
                    for event in window.events() {
                        match event.to_option() {
                            EventOption::Mouse(mouse_event) => {
                                draw_chooser(&mut window, &packages, mouse_event.x, mouse_event.y);

                                if mouse_event.left_button {
                                    let mut y = 0;
                                    for package in packages.iter() {
                                        if package.icon.has_data() {
                                            if mouse_event.y >= y && mouse_event.y < y + 32 {
                                                if let Err(err) = Command::new(&package.binary).arg(path).spawn() {
                                                    println!("{}: Failed to launch: {}", package.binary, err);
                                                }
                                                break 'choosing;
                                            }
                                            y += 32;
                                        }
                                    }
                                }
                            },
                            EventOption::Quit(_) => break 'choosing,
                            _ => ()
                        }
                    }

                    thread::yield_now();
                }
            } else if let Some(package) = packages.get(0) {
                if let Err(err) = Command::new(&package.binary).arg(&path).spawn() {
                    println!("launcher: failed to launch '{}': {}", package.binary, err);
                }
            } else {
                println!("launcher: no application found for '{}'", path);
            }
        }
    } else {
        let packages = get_packages();

        let shutdown = BmpFile::from_path("/ui/actions/system-shutdown.bmp");
        if ! shutdown.has_data() {
            println!("launcher: failed to read shutdown icon");
        }

        let mut window = Window::new(0, 768 - 32, 1024, 32, "").unwrap();

        let mut selected = -1;

        draw(&mut window, &packages, &shutdown, selected);
        'running: loop {
            for event in window.events() {
                match event.to_option() {
                    EventOption::Mouse(mouse_event) => {
                        let mut now_selected = -1;

                        {
                            let mut x = 0;
                            let mut i = 0;
                            for package in packages.iter() {
                                if package.icon.has_data() {
                                    let y = window.height() as i32 - package.icon.height() as i32;
                                    if mouse_event.y >= y && mouse_event.x >= x &&
                                       mouse_event.x < x + package.icon.width() as i32 {
                                        now_selected = i;
                                    }
                                    x = x + package.icon.width() as i32;
                                    i += 1;
                                }
                            }

                            if shutdown.has_data() {
                                x = window.width() as i32 - shutdown.width() as i32;
                                let y = window.height() as i32 - shutdown.height() as i32;
                                if mouse_event.y >= y && mouse_event.x >= x &&
                                   mouse_event.x < x + shutdown.width() as i32 {
                                       now_selected = i;
                                }
                                i += 1;
                            }
                        }

                        if now_selected != selected {
                            selected = now_selected;
                            draw(&mut window, &packages, &shutdown, selected);
                        }

                        if mouse_event.left_button {
                            let mut i = 0;
                            for package in packages.iter() {
                                if package.icon.has_data() {
                                    if i == selected {
                                        if let Err(err) = Command::new(&package.binary).spawn() {
                                            println!("{}: Failed to launch: {}", package.binary, err);
                                        }
                                    }
                                    i += 1;
                                }
                            }

                            if shutdown.has_data() {
                                if i == selected {
                                       File::create("acpi:off");
                                }
                                i += 1;
                            }
                        }
                    },
                    EventOption::Quit(_) => break 'running,
                    _ => ()
                }
            }

            thread::yield_now();
        }
    }
}
