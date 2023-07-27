use egui::{RichText, Ui, Widget};
use itertools::Itertools;
use packet_inspector::PacketState;

use crate::tri_checkbox::{TriCheckbox, TriCheckboxState};

use super::{SharedState, Tab, View};

pub struct Filter {}

impl Tab for Filter {
    fn new() -> Self {
        Self {}
    }

    fn name(&self) -> &'static str {
        "Filters"
    }
}

impl View for Filter {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            if ui.button("x").clicked() {
                state.packet_search.clear();
            }
            ui.text_edit_singleline(&mut state.packet_search);
        });

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(false)
            .show(ui, |ui| {
                if draw_packet_list(ui, state, PacketState::Handshaking) > 0 {
                    ui.separator();
                }
                if draw_packet_list(ui, state, PacketState::Status) > 0 {
                    ui.separator();
                }
                if draw_packet_list(ui, state, PacketState::Login) > 0 {
                    ui.separator();
                }
                draw_packet_list(ui, state, PacketState::Play);
            });
    }
}

fn get_checkbox_state(state: &SharedState, packet_state: PacketState) -> TriCheckboxState {
    let mut p_enabled = 0;
    let mut disabled = 0;
    for (_, enabled) in state
        .packet_filter
        .iter()
        .filter(|(p, _)| p.state == packet_state)
    {
        if *enabled {
            p_enabled += 1;
        } else {
            disabled += 1;
        }
    }
    if p_enabled > 0 && disabled == 0 {
        TriCheckboxState::Enabled
    } else if p_enabled > 0 && disabled > 0 {
        TriCheckboxState::Partial
    } else {
        TriCheckboxState::Disabled
    }
}

fn draw_packet_list(ui: &mut Ui, state: &mut SharedState, packet_state: PacketState) -> usize {
    let title = match packet_state {
        PacketState::Handshaking => "Handshaking",
        PacketState::Status => "Status",
        PacketState::Login => "Login",
        PacketState::Play => "Play",
    };

    let search = state.packet_search.to_lowercase();

    let count = state
        .packet_filter
        .iter_mut()
        .filter(|(p, _)| p.state == packet_state && p.name.to_lowercase().contains(&search))
        .count();

    let count_enabled = state
        .packet_filter
        .iter_mut()
        .filter(|(p, enabled)| {
            p.state == packet_state && p.name.to_lowercase().contains(&search) && **enabled
        })
        .count();

    if count == 0 {
        return 0;
    }

    let mut checkbox = get_checkbox_state(state, packet_state);
    if TriCheckbox::new(&mut checkbox, RichText::new(title).heading().strong())
        .ui(ui)
        .changed()
    {
        for (_, enabled) in state
            .packet_filter
            .iter_mut()
            .filter(|(p, _)| p.state == packet_state && p.name.to_lowercase().contains(&search))
        {
            if count == count_enabled || count_enabled == 0 {
                *enabled = !*enabled;
                continue;
            }

            *enabled = (count / 2) <= count_enabled;
        }
    }

    for (p, enabled) in state
        .packet_filter
        .iter_mut()
        .filter(|(p, _)| p.state == packet_state && p.name.to_lowercase().contains(&search))
        .sorted_by(|(a, _), (b, _)| {
            a.id.cmp(&b.id)
                .then((a.side as usize).cmp(&(b.side as usize)))
        })
    {
        ui.checkbox(enabled, format!("[0x{:0>2X}] {}", p.id, p.name));
    }

    count
}
