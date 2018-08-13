fn main() {
    let arg = match std::env::args().nth(1) {
        Some(a) => a,
        None => {
            print_help();
            return;
        }
    };
    match arg.as_str() {
        "sleep" => {
            for i in 0..30 {
                sleep(1);
                println!("{:?}", i + 1);
            }
        }
        "abort" => std::process::exit(1),
        "sleep_abort" => {
            sleep(1);
            std::process::exit(1);
        }
        "finish" => {}
        "sleep_finish" => sleep(1),
        _ => print_help(),
    }
}

fn sleep(time: u64) {
    std::thread::sleep(std::time::Duration::from_secs(time));
}

fn print_help() {
    println!("Usage: test_tool <arg>");
    println!(
        "- sleep: sleep for 1 second intervals, printing a counter every second, for 30 seconds"
    );
    println!("- abort: abort immediately (non-0 error code)");
    println!("- sleep_abort: sleep for 1 second, then abort (non-0 error code)");
    println!("- finish: finish immediately (0 error code)");
    println!("- sleep_finish: sleep for 1 second, then finish (0 error code)");
}
