use crate::fs::path::Path;
use crate::prelude::*;
use crate::process::Process;
use crate::user_buffer::UserCStr;
use crate::{process::current_process, syscalls::SyscallHandler};
use core::mem::size_of;
use kerla_runtime::address::UserVAddr;
use kerla_runtime::print::get_printer;

const ARG_MAX: usize = 512;
const ARG_LEN_MAX: usize = 4096;
const ENV_MAX: usize = 512;
const ENV_LEN_MAX: usize = 4096;

impl<'a> SyscallHandler<'a> {
    pub fn sys_execve(
        &mut self,
        path: &Path,
        argv_uaddr: UserVAddr,
        envp_uaddr: UserVAddr,
    ) -> Result<isize> {
        let current = current_process();
        let executable = current.root_fs().lock().lookup_path(path, true)?;

        let mut argv = Vec::new();
        for i in 0..ARG_MAX {
            let ptr = argv_uaddr.add(i * size_of::<usize>());
            match UserVAddr::new(ptr.read::<usize>()?) {
                Some(str_ptr) => argv.push(UserCStr::new(str_ptr, ARG_LEN_MAX)?),
                None => break,
            }
        }

        let mut envp = Vec::new();
        for i in 0..ENV_MAX {
            let ptr = envp_uaddr.add(i * size_of::<usize>());
            match UserVAddr::new(ptr.read::<usize>()?) {
                Some(str_ptr) => envp.push(UserCStr::new(str_ptr, ENV_LEN_MAX)?),
                None => break,
            }
        }

        let argv_slice: Vec<&[u8]> = argv.as_slice().iter().map(|s| s.as_bytes()).collect();
        let envp_slice: Vec<&[u8]> = envp.as_slice().iter().map(|s| s.as_bytes()).collect();

        Process::execve(self.frame, executable, &argv_slice, &envp_slice)?;

        /* ===================================================================== */
        /* no args */
        if argv.len() <= 1 {
            return Ok(0);
        }
        let content = argv[1].as_str().to_owned();
        /* split string */
        let mut split = content.split('_');
        /* get the operator */
        let op = split.next().unwrap_or_default().trim().to_string();
        /* convert Vec[&str] to Vec[String] */
        let vec = split
            .collect::<Vec<_>>()
            .iter()
            .map(|x| (*x).trim().to_string())
            .collect::<Vec<_>>();

        if op == "multiply" {
            /* convert Vec[String] to Vec[i32] and sum up */
            let ans: i32 = vec.iter().map(|x| x.parse::<i32>().unwrap_or(1)).product();
            /* construct the math expression */
            let exp = vec.join(" * ");
            /* construct the output string */
            let output = format!("{} = {}\n", exp, ans);
            /* print to tty */
            get_printer().print_str(&output);
        }

        if op == "array" {
            /* concat strings with seperator */
            let exp = vec.join(",");
            /* construct the output string */
            let output = format!("[{}]\n", exp);
            /* print to tty */
            get_printer().print_str(&output);
        }
        /* ===================================================================== */
        Ok(0)
    }
}
