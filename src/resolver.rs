use std::collections::HashSet;

/// Represents a protected system port and its service description.
pub struct ProtectedPort {
    pub port: u16,
    pub service: &'static str,
}

pub const PROTECTED_PORTS: &[ProtectedPort] = &[
    ProtectedPort {
        port: 22,
        service: "SSH (Secure Shell)",
    },
    ProtectedPort {
        port: 53,
        service: "DNS (Domain Name System)",
    },
    ProtectedPort {
        port: 67,
        service: "DHCP Server",
    },
    ProtectedPort {
        port: 68,
        service: "DHCP Client",
    },
    ProtectedPort {
        port: 123,
        service: "NTP (Network Time Protocol)",
    },
    ProtectedPort {
        port: 161,
        service: "SNMP (Network Management)",
    },
];

pub fn get_protection(port: u16) -> Option<&'static str> {
    PROTECTED_PORTS
        .iter()
        .find(|p| p.port == port)
        .map(|p| p.service)
}

/// Known framework and service port aliases.
pub const ALIASES: &[(&str, &[u16])] = &[
    ("vite", &[5173]),
    ("next", &[3000]),
    ("react", &[3000]),
    ("remix", &[3000]),
    ("astro", &[3000, 4321]),
    ("svelte", &[5173]),
    ("vue", &[5173, 8080]),
    ("nuxt", &[3000]),
    ("angular", &[4200]),
    ("rails", &[3000]),
    ("django", &[8000]),
    ("fastapi", &[8000]),
    ("flask", &[5000]),
    ("express", &[3000]),
    ("spring", &[8080]),
    ("webpack", &[8080]),
    ("tomcat", &[8080]),
    ("storybook", &[6006]),
    ("graphql", &[4000]),
    ("postgres", &[5432]),
    ("pgsql", &[5432]),
    ("psql", &[5432]),
    ("mysql", &[3306]),
    ("mariadb", &[3306]),
    ("redis", &[6379]),
    ("mongo", &[27017]),
    ("mongodb", &[27017]),
    ("elastic", &[9200]),
    ("elasticsearch", &[9200]),
];

/// Resolves a single target string (alias, range, or numeric port) into a set of ports.
pub fn resolve_target(target: &str) -> Result<Vec<u16>, String> {
    let lower = target.trim().to_lowercase();

    // 1. Check known alias
    for (alias, ports) in ALIASES {
        if &lower == alias {
            return Ok(ports.to_vec());
        }
    }

    // 2. Check range syntax (e.g. 3000..3005 or 3000-3005)
    if let Some((start_s, end_s)) = lower.split_once("..").or_else(|| lower.split_once('-')) {
        let start: u16 = start_s
            .trim()
            .parse()
            .map_err(|_| format!("Invalid start port in range: '{start_s}'"))?;
        let end: u16 = end_s
            .trim()
            .parse()
            .map_err(|_| format!("Invalid end port in range: '{end_s}'"))?;

        if start > end {
            return Err(format!(
                "Invalid range: start port {start} is greater than end port {end}"
            ));
        }

        if end - start > 100 {
            return Err(format!(
                "Port range too large: spans {} ports. Max allowed range is 100 ports.",
                end - start + 1
            ));
        }

        return Ok((start..=end).collect());
    }

    // 3. Check direct numeric port
    match lower.parse::<u16>() {
        Ok(port) => {
            if port == 0 {
                Err("Port 0 is reserved and cannot be nuked.".to_string())
            } else {
                Ok(vec![port])
            }
        }
        Err(_) => Err(format!(
            "Unrecognized port, alias, or range: '{target}'. Run 'pnuke --aliases' to see supported aliases."
        )),
    }
}

/// Resolves multiple targets, deduplicating ports while preserving ascending order.
pub fn resolve_all_targets(targets: &[String]) -> Result<Vec<u16>, String> {
    let mut ports_set = HashSet::new();

    for t in targets {
        let resolved = resolve_target(t)?;
        for p in resolved {
            ports_set.insert(p);
        }
    }

    let mut ports: Vec<u16> = ports_set.into_iter().collect();
    ports.sort_unstable();
    Ok(ports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_resolution() {
        assert_eq!(resolve_target("vite").unwrap(), vec![5173]);
        assert_eq!(resolve_target("next").unwrap(), vec![3000]);
        assert_eq!(resolve_target("postgres").unwrap(), vec![5432]);
    }

    #[test]
    fn test_range_resolution() {
        assert_eq!(
            resolve_target("3000..3003").unwrap(),
            vec![3000, 3001, 3002, 3003]
        );
        assert_eq!(resolve_target("8000-8002").unwrap(), vec![8000, 8001, 8002]);
    }

    #[test]
    fn test_invalid_range() {
        assert!(resolve_target("3005..3000").is_err());
        assert!(resolve_target("1..500").is_err());
    }

    #[test]
    fn test_protected_ports() {
        assert!(get_protection(22).is_some());
        assert!(get_protection(53).is_some());
        assert!(get_protection(3000).is_none());
    }
}
