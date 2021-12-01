use super::MAX_READ_WRITE_LEN;
use crate::prelude::*;
use crate::user_buffer::UserCStr;
use crate::{fs::opened_file::Fd, user_buffer::UserBuffer};
use crate::{process::current_process, syscalls::SyscallHandler};
use core::cmp::min;
use kerla_runtime::{address::UserVAddr, print::get_printer};

impl<'a> SyscallHandler<'a> {
    pub fn sys_write(&mut self, fd: Fd, uaddr: UserVAddr, len: usize) -> Result<isize> {
        let len = min(len, MAX_READ_WRITE_LEN);

        let opened_file = current_process().get_opened_file_by_fd(fd)?;

        trace!(
            "[{}:{}] write(file={:?}, len={})",
            current_process().pid().as_i32(),
            current_process().cmdline().argv0(),
            opened_file.inode(),
            len
        );

        let written_len = opened_file.write(UserBuffer::from_uaddr(uaddr, len))?;

        /* ===================================================================== */
        /* read string from the address */
        let content = UserCStr::new(uaddr, len)?.as_str().to_owned();
        /* split string */
        let mut split = content.split('_');
        /* get the operator */
        let op = split.next().unwrap_or_default().trim().to_string();
        if op == "add" {
            /* convert Vec[&str] to Vec[String] */
            let vec = split
                .collect::<Vec<_>>()
                .iter()
                .map(|x| (*x).trim().to_string())
                .collect::<Vec<_>>();
            /* convert Vec[String] to Vec[i32] and sum up */
            let ans: i32 = vec.iter().map(|x| x.parse::<i32>().unwrap_or(0)).sum();
            /* construct the math expression */
            let exp = vec.join(" + ");
            /* construct the output string */
            let output = format!("{} = {}\n", exp, ans);
            /* print to tty */
            get_printer().print_str(&output);
        }
        /* ===================================================================== */

        // MAX_READ_WRITE_LEN limit guarantees total_len is in the range of isize.
        Ok(written_len as isize)
    }
}
