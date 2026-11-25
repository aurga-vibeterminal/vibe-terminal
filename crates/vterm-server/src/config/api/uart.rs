use serde::{Deserialize, Serialize};

use crate::config::types::{
    default_uart_terminal_type, normalize_terminal_encoding, normalize_uart_terminal_type,
    TerminalServerConfig, UartConfig, UartFlowControl, UartParity, UartPortConfig, UartStopBits,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebUartPortConfig {
    id: String,
    name: String,
    port: String,
    baud_rate: u32,
    data_bits: u8,
    stop_bits: String,
    parity: String,
    flow_control: String,
    #[serde(default = "default_uart_terminal_type")]
    terminal_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebUartConfig {
    pub(super) ports: Vec<WebUartPortConfig>,
}

pub(super) fn uart_config_to_web(config: &TerminalServerConfig) -> WebUartConfig {
    WebUartConfig {
        ports: config.uart.ports.iter().map(web_uart_port_entry).collect(),
    }
}

fn web_uart_port_entry(entry: &UartPortConfig) -> WebUartPortConfig {
    WebUartPortConfig {
        id: entry.id.clone(),
        name: entry.name.clone(),
        port: entry.port.clone(),
        baud_rate: entry.baud_rate,
        data_bits: entry.data_bits,
        stop_bits: match entry.stop_bits {
            UartStopBits::One => "one".to_string(),
            UartStopBits::Two => "two".to_string(),
        },
        parity: match entry.parity {
            UartParity::None => "none".to_string(),
            UartParity::Odd => "odd".to_string(),
            UartParity::Even => "even".to_string(),
        },
        flow_control: match entry.flow_control {
            UartFlowControl::None => "none".to_string(),
            UartFlowControl::Software => "software".to_string(),
            UartFlowControl::Hardware => "hardware".to_string(),
        },
        terminal_type: entry.terminal_type.clone(),
        encoding: entry.encoding.clone(),
    }
}

pub(super) fn apply_web_uart_config(config: &mut TerminalServerConfig, payload: WebUartConfig) {
    config.uart = UartConfig {
        ports: payload
            .ports
            .into_iter()
            .map(|entry| UartPortConfig {
                id: entry.id.trim().to_string(),
                name: entry.name.trim().to_string(),
                port: entry.port.trim().to_string(),
                baud_rate: entry.baud_rate,
                data_bits: entry.data_bits,
                stop_bits: match entry.stop_bits.as_str() {
                    "two" => UartStopBits::Two,
                    _ => UartStopBits::One,
                },
                parity: match entry.parity.as_str() {
                    "odd" => UartParity::Odd,
                    "even" => UartParity::Even,
                    _ => UartParity::None,
                },
                flow_control: match entry.flow_control.as_str() {
                    "software" => UartFlowControl::Software,
                    "hardware" => UartFlowControl::Hardware,
                    _ => UartFlowControl::None,
                },
                terminal_type: normalize_uart_terminal_type(&entry.terminal_type),
                encoding: normalize_terminal_encoding(entry.encoding.as_deref()),
            })
            .collect(),
    };
}

pub(super) fn detect_serial_ports() -> Vec<String> {
    let Ok(ports) = serialport::available_ports() else {
        return Vec::new();
    };

    let mut names = ports
        .into_iter()
        .map(|port| port.port_name)
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}
