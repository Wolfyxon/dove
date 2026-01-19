use aes_gcm::aead::rand_core::{self};
use rand::{SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha256};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks};

pub mod aes256;

pub fn get_machine_summary() -> String {
    let cpu_refresh = CpuRefreshKind::nothing().with_frequency();
    let mem_refresh = MemoryRefreshKind::nothing().with_ram();
    let sys_refresh = sysinfo::RefreshKind::nothing()
        .with_cpu(cpu_refresh)
        .with_memory(mem_refresh);

    let sys = sysinfo::System::new_with_specifics(sys_refresh);
    let components = sysinfo::Components::new_with_refreshed_list();
    let network_interfaces = Networks::new_with_refreshed_list();
    
    let arch = sysinfo::System::cpu_arch();
    let sys_name = sysinfo::System::name().unwrap_or("unknown".to_string());
    let host_name = sysinfo::System::name().unwrap_or("unknown".to_string());
    let distro = sysinfo::System::distribution_id();
    let total_mem = sys.total_memory();
    let component_count = components.len();

    let mut total_critical_temp: f32 = 0.0;
    let mut mac_sum: u64 = 0;

    for (_name, data) in &network_interfaces {
        let mac = data.mac_address();
        
        for byte in mac.0 {
            mac_sum += byte as u64;
        }
    }

    for component in &components {
        
        if let Some(temp) = component.critical() {
            if !temp.is_nan() {
                total_critical_temp += temp;
            }
        }
    }

    format!("{arch}|{sys_name}|{host_name}|{distro}|{total_mem}|{component_count}|{total_critical_temp}|{mac_sum}")
}

pub fn rng_from_string(seed: impl Into<String>) -> StdRng {    
    let hash = Sha256::digest(seed.into());
    let seed_buf: [u8; 32] = hash.into();

    StdRng::from_seed(seed_buf)
}

pub fn get_machine_id_rng() -> StdRng {
    rng_from_string(get_machine_summary())
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use rand::RngCore;

    use crate::crypto::{get_machine_summary, rng_from_string};

    #[test]
    fn test_rng() {
        let seed = "Hello there this is a test seed";
        
        let mut rng1 = rng_from_string(seed);
        let mut rng2 = rng_from_string(seed);

        const LEN: usize = 10;

        let mut buf1: [u8; LEN] = [0; LEN];
        let mut buf2: [u8; LEN] = [0; LEN];
        
        rng1.fill_bytes(&mut buf1);
        rng2.fill_bytes(&mut buf2);
        
        println!("A: {:?}", buf1);
        println!("B: {:?}", buf2);

        for i in 0..LEN {
            assert_eq!(buf1[i], buf2[i]);
        }
    }

    #[test]
    fn test_machine_summary() {
        for _i in 0..5 {
            let summ1 = get_machine_summary();
            thread::sleep(Duration::from_millis(10));
            
            let summ2 = get_machine_summary();
        
            assert_eq!(summ1, summ2);
        }
    }
}
