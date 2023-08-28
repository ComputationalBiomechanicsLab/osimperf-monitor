use nix::fcntl::{flock, FlockArg};
use nix::unistd::close;
use std::fs::File;
use std::os::unix::io::AsRawFd;

pub struct CompilationStatus {
    pub percentage: f64,
    pub process: String,
}

pub enum Status {
    Compiling(CompilationStatus),
    Failed,
    Done,
}

pub struct CompilationNode {
    repo: Repository,
    status: Status,
}

pub fn shared_read() {
    // Open the file for locking
    let file = File::open("file_to_lock.txt").expect("Failed to open file");

    // Acquire a lock on the file
    let lock_result = flock(
        file.as_raw_fd(),
        FlockArg::LockSharedNonblock,
    );

    match lock_result {
        Ok(_) => {
            println!("Lock acquired. Do your work here.");

            // Simulate some work being done
            std::thread::sleep(std::time::Duration::from_secs(5));

            // Release the lock
            let unlock_result = flock(
                file.as_raw_fd(),
                FlockArg::Unlock,
            );

            if unlock_result.is_err() {
                eprintln!("Failed to release lock");
            }
        }
        Err(_) => {
            eprintln!("Failed to acquire lock");
        }
    }

    // Close the file explicitly (not required, but good practice)
    close(file.as_raw_fd()).expect("Failed to close file");
}
