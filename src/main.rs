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
    port: i32,
}

fn main() {
    let args = Cli::from_args();
    if let Ok(pids) = listpids(ProcType::ProcAllPIDS) {
        let mut found_pid = 0;
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
                                            let mut port = 0;
                                            port |= info.tcpsi_ini.insi_lport >> 8 & 0x00ff;
                                            port |= info.tcpsi_ini.insi_lport << 8 & 0xff00;

                                            if args.port == port {
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
        if found_pid > 0 {
            let sig = if args.force {
                nix::sys::signal::SIGKILL
            } else {
                nix::sys::signal::SIGINT
            };
            match nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(found_pid.try_into().unwrap()),
                sig,
            ) {
                Ok(_) => println!("Killed process {} with {}", found_pid, sig),
                Err(err) => println!(
                    "Failed to kill process {} with {}: {:?}",
                    found_pid, sig, err
                ),
            }
        } else {
            println!("Unable to find a process using port {}", args.port);
        }
    }
}
