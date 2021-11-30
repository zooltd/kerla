use super::MAX_READ_WRITE_LEN;
use crate::prelude::*;
use crate::{fs::opened_file::Fd, user_buffer::UserBuffer};
use crate::{process::current_process, syscalls::SyscallHandler};
use core::cmp::min;
use kerla_runtime::{address::UserVAddr, print::get_printer};

impl<'a> SyscallHandler<'a> {
    pub fn sys_write(&mut self, fd: Fd, uaddr: UserVAddr, len: usize) -> Result<isize> {
        let len = min(len, MAX_READ_WRITE_LEN);

        let mut tmp = vec![0; len];
        let copied_len = uaddr.read_cstr(tmp.as_mut_slice())?;
        let content = core::str::from_utf8(&tmp[..copied_len])
        .map_err(|_| Error::new(Errno::EINVAL))?
        .to_owned();

        get_printer().print_str(&content);

        let opened_file = current_process().get_opened_file_by_fd(fd)?;
        trace!(
            "[{}:{}] write(file={:?}, len={})",
            current_process().pid().as_i32(),
            current_process().cmdline().argv0(),
            opened_file.inode(),
            len
        );

        let written_len = opened_file.write(UserBuffer::from_uaddr(uaddr, len))?;

        // MAX_READ_WRITE_LEN limit guarantees total_len is in the range of isize.
        Ok(written_len as isize)
    }
}
