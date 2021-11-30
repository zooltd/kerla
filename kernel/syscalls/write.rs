use super::MAX_READ_WRITE_LEN;
use crate::prelude::*;
use crate::user_buffer::UserCStr;
use crate::{fs::opened_file::Fd, user_buffer::UserBuffer};
use crate::{process::current_process, syscalls::SyscallHandler};
use core::cmp::min;
use core::mem::size_of;
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

        /* ===================================================================== */
        let written_len = opened_file.write(UserBuffer::from_uaddr(uaddr, len))?;

        let content = UserCStr::new(uaddr, len)?.as_str().to_owned();

        let mut split = content.split('_');

        let instruction: (String, String, String) = (
            split.next().unwrap().trim().to_string(),
            split.next().unwrap().trim().to_string(),
            split.next().unwrap().trim().to_string(),
        );

        if instruction.0 == "add" {
            let num1 = instruction.1.to_owned().parse::<i32>().unwrap();
            let num2 = instruction.2.to_owned().parse::<i32>().unwrap();
            let ans = num1 + num2;
            let output = format!("{} + {} = {}\n", num1, num2, ans);
            get_printer().print_str(&output);
        }
        /* ===================================================================== */

        // MAX_READ_WRITE_LEN limit guarantees total_len is in the range of isize.
        Ok(written_len as isize)
    }
}
