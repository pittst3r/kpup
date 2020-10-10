use libproc::libproc::bsd_info::BSDInfo;
use libproc::libproc::file_info::{pidfdinfo, ListFDs, ProcFDType};
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

    if pid > 0 {
        let sig = if args.force {
            nix::sys::signal::SIGKILL
        } else {
            nix::sys::signal::SIGINT
        };
        match nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), sig) {
            Ok(_) => println!("Killed process {} with {}", pid, sig),
            Err(err) => println!("Failed to kill process {} with {}: {:?}", pid, sig, err),
        }
    } else {
        println!("Unable to find a process using port {}", args.port);
    }
}

fn find_pid(port: u32) -> i32 {
    let given_port: i32 = match port.try_into() {
        Ok(p) => p,
        Err(_) => panic!("Unable to use given port for search"),
    };
    let mut found_pid = 0;

    if let Ok(pids) = listpids(ProcType::ProcAllPIDS) {
        for pid in pids {
            if let Ok(info) = pidinfo::<BSDInfo>(pid.try_into().unwrap(), 0) {
                if let Ok(fds) =
                    listpidinfo::<ListFDs>(pid.try_into().unwrap(), info.pbi_nfiles as usize)
                {
                    for fd in &fds {
                        match fd.proc_fdtype.into() {
                            ProcFDType::Socket => {
                                if let Ok(socket) =
                                    pidfdinfo::<SocketFDInfo>(pid.try_into().unwrap(), fd.proc_fd)
                                {
                                    match socket.psi.soi_kind.into() {
                                        SocketInfoKind::Tcp => {
                                            // access to the member of `soi_proto` is unsafe because of union type.
                                            let info = unsafe { socket.psi.soi_proto.pri_tcp };
                                            // change endian and cut off because insi_lport is network endian and 16bit width.
                                            let mut current_port = 0;
                                            current_port |= info.tcpsi_ini.insi_lport >> 8 & 0x00ff;
                                            current_port |= info.tcpsi_ini.insi_lport << 8 & 0xff00;

                                            if given_port == current_port {
                                                found_pid = pid;
                                                break;
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }

            if found_pid > 0 {
                break;
            }
        }
    }

    return found_pid.try_into().unwrap();
}
