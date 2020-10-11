use libproc::libproc::bsd_info::BSDInfo;
use libproc::libproc::file_info::{pidfdinfo, ListFDs, ProcFDInfo, ProcFDType};
use libproc::libproc::net_info::{SocketFDInfo, SocketInfoKind};
use libproc::libproc::proc_pid::{listpidinfo, listpids, pidinfo, ProcType};
use nix;
use std::convert::TryInto;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "kpup", about = "Kill Process Using Port")]
struct Cli {
    #[structopt(
        short,
        long,
        takes_value = false,
        help = "Send SIGKILL instead of SIGINT"
    )]
    force: bool,

    #[structopt(required = true, help = "the port on which the process is listening")]
    port: u32,
}

fn main() {
    let args = Cli::from_args();
    let pid = find_pid(args.port);
    let signal = if args.force {
        nix::sys::signal::SIGKILL
    } else {
        nix::sys::signal::SIGINT
    };

    match pid {
        Ok(p) => match nix::sys::signal::kill(nix::unistd::Pid::from_raw(p), signal) {
            Ok(_) => println!("Killed process {} with {}", p, signal),
            Err(err) => println!("Failed to kill process {} with {}: {:?}", p, signal, err),
        },
        Err(reason) => println!("{}", reason),
    }
}

fn find_pid(port: u32) -> Result<i32, String> {
    let given_port: i32 = match port.try_into() {
        Ok(p) => p,
        Err(_) => return Err(String::from("Unable to use given port for search")),
    };
    if let Ok(pids) = listpids(ProcType::ProcAllPIDS) {
        for pid in pids {
            if let Ok(pid) = pid.try_into() {
                if let Ok(info) = pidinfo::<BSDInfo>(pid, 0) {
                    if let Ok(fds) = listpidinfo::<ListFDs>(pid, info.pbi_nfiles as usize) {
                        match search_fds(given_port, pid, fds) {
                            Ok(found_pid) => return Ok(found_pid),
                            Err(_) => (),
                        }
                    }
                }
            }
        }
    }

    Err(String::from("Could not find process using given port"))
}

fn search_fds(port: i32, pid: i32, fds: std::vec::Vec<ProcFDInfo>) -> Result<i32, ()> {
    for fd in fds {
        match fd.proc_fdtype.into() {
            ProcFDType::Socket => {
                if let Ok(socket) = pidfdinfo::<SocketFDInfo>(pid, fd.proc_fd) {
                    match socket.psi.soi_kind.into() {
                        SocketInfoKind::Tcp => {
                            // access to the member of `soi_proto` is unsafe because of union type.
                            let info = unsafe { socket.psi.soi_proto.pri_tcp };
                            // change endian and cut off because insi_lport is network endian and 16bit width.
                            let mut current_port = 0;
                            current_port |= info.tcpsi_ini.insi_lport >> 8 & 0x00ff;
                            current_port |= info.tcpsi_ini.insi_lport << 8 & 0xff00;

                            if port == current_port {
                                return Ok(pid);
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    Err(())
}
