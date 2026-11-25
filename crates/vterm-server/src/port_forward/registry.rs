use std::collections::HashMap;

use vterm_common::PortForwardInfo;

use super::types::{PortForwardRuntime, PortForwardTarget};

#[derive(Default)]
pub(crate) struct PortForwardRegistry {
    by_target: HashMap<PortForwardTarget, PortForwardRuntime>,
    target_by_proxy_port: HashMap<u16, PortForwardTarget>,
}

impl PortForwardRegistry {
    pub(crate) fn get_by_target(&self, target: PortForwardTarget) -> Option<&PortForwardRuntime> {
        self.by_target.get(&target)
    }

    pub(crate) fn contains_proxy_port(&self, proxy_port: u16) -> bool {
        self.target_by_proxy_port.contains_key(&proxy_port)
    }

    pub(crate) fn get_by_proxy_port(&self, proxy_port: u16) -> Option<&PortForwardRuntime> {
        let target = self.target_by_proxy_port.get(&proxy_port)?;
        self.by_target.get(target)
    }

    pub(crate) fn remove_by_proxy_port(&mut self, proxy_port: u16) -> Option<PortForwardRuntime> {
        let target = self.target_by_proxy_port.remove(&proxy_port)?;
        self.by_target.remove(&target)
    }

    pub(crate) fn insert(&mut self, runtime: PortForwardRuntime) {
        self.target_by_proxy_port
            .insert(runtime.proxy_port, runtime.target);
        self.by_target.insert(runtime.target, runtime);
    }

    pub(crate) fn list(&self) -> Vec<PortForwardInfo> {
        let mut proxies = self
            .by_target
            .values()
            .map(PortForwardRuntime::info)
            .collect::<Vec<_>>();
        proxies.sort_by_key(|proxy| proxy.proxy_port);
        proxies
    }

    pub(crate) fn take_all(&mut self) -> Vec<PortForwardRuntime> {
        self.target_by_proxy_port.clear();
        self.by_target.drain().map(|(_, runtime)| runtime).collect()
    }
}
