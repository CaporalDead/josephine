mod battery;
mod cpu;
mod disk;
mod filesystem;
mod inode;
mod kernel;
mod memory;
mod network;
mod reboot;
mod security;
mod smart;
mod systemd;
mod temperature;
mod timesync;
mod updates;

pub use battery::BatteryCheck;
pub use cpu::CpuCheck;
pub use disk::DiskCheck;
pub use filesystem::FilesystemCheck;
pub use inode::InodeCheck;
pub use kernel::KernelCheck;
pub use memory::MemoryCheck;
pub use network::NetworkCheck;
pub use reboot::RebootCheck;
pub use security::SecurityCheck;
pub use smart::SmartCheck;
pub use systemd::SystemdCheck;
pub use temperature::TemperatureCheck;
pub use timesync::TimesyncCheck;
pub use updates::UpdatesCheck;

use crate::check::Check;
use crate::config::ChecksConfig;

pub fn build_checks(config: &ChecksConfig) -> Vec<Box<dyn Check>> {
    let mut checks: Vec<Box<dyn Check>> = Vec::new();

    if config.cpu.enabled {
        checks.push(Box::new(CpuCheck::new(config.cpu.clone())));
    }
    if config.memory.enabled {
        checks.push(Box::new(MemoryCheck::new(config.memory.clone())));
    }
    if config.disk.enabled {
        checks.push(Box::new(DiskCheck::new(config.disk.clone())));
    }
    if config.temperature.enabled {
        checks.push(Box::new(TemperatureCheck::new(config.temperature.clone())));
    }
    if config.systemd.enabled {
        checks.push(Box::new(SystemdCheck::new(config.systemd.clone())));
    }
    if config.updates.enabled {
        checks.push(Box::new(UpdatesCheck::new(config.updates.clone())));
    }
    if config.network.enabled {
        checks.push(Box::new(NetworkCheck::new(config.network.clone())));
    }
    if config.battery.enabled {
        checks.push(Box::new(BatteryCheck::new(config.battery.clone())));
    }
    if config.inode.enabled {
        checks.push(Box::new(InodeCheck::new(config.inode.clone())));
    }
    if config.smart.enabled {
        checks.push(Box::new(SmartCheck::new(config.smart.clone())));
    }
    if config.kernel.enabled {
        checks.push(Box::new(KernelCheck::new(config.kernel.clone())));
    }
    if config.filesystem.enabled {
        checks.push(Box::new(FilesystemCheck::new(config.filesystem.clone())));
    }
    if config.timesync.enabled {
        checks.push(Box::new(TimesyncCheck::new(config.timesync.clone())));
    }
    if config.security.enabled {
        checks.push(Box::new(SecurityCheck::new(config.security.clone())));
    }
    if config.reboot.enabled {
        checks.push(Box::new(RebootCheck::new(config.reboot.clone())));
    }

    checks
}

pub fn interval_for_check(name: &str, config: &ChecksConfig) -> u64 {
    match name {
        "cpu" => config.cpu.interval_secs,
        "memory" => config.memory.interval_secs,
        "disk" => config.disk.interval_secs,
        "temperature" => config.temperature.interval_secs,
        "systemd" => config.systemd.interval_secs,
        "updates" => config.updates.interval_secs,
        "network" => config.network.interval_secs,
        "battery" => config.battery.interval_secs,
        "inode" => config.inode.interval_secs,
        "smart" => config.smart.interval_secs,
        "kernel" => config.kernel.interval_secs,
        "filesystem" => config.filesystem.interval_secs,
        "timesync" => config.timesync.interval_secs,
        "security" => config.security.interval_secs,
        "reboot" => config.reboot.interval_secs,
        _ => 60,
    }
}
