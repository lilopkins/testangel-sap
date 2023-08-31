use std::{fs, sync::Mutex};

use lazy_static::lazy_static;
use sap_scripting::*;
use testangel_engine::*;

#[derive(Default)]
struct State {
    com_instance: Option<SAPComInstance>,
    session: Option<GuiSession>,
}
unsafe impl Send for State {}

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, Mutex<State>>> = Mutex::new(Engine::new("SAP")
    .with_instruction(
        Instruction::new(
            "sap-connect",
            "Connect to Open Instance",
            "Connect to an SAP instance that the user already has open.\nIf they have multiple open, this will give access to any of the open windows (although most instructions use the main window).\nThis will do nothing if we already hold a connection.",
        ),
        |state: &mut Mutex<State>, _params, _output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");

            if state.com_instance.is_none() {
                match connect(&mut *state) {
                    Err(e) => {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: e,
                        })
                    }
                    Ok(_) => (),
                }
            }

            None
        })
    .with_instruction(
        Instruction::new(
            "sap-run-transaction",
            "Run Transaction",
            "Run a transaction.",
        )
        .with_parameter("tcode", "Transaction Code", ParameterKind::String),
        |state, params, _output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");
            let tcode = params["tcode"].value_string();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Err(e) = session.start_transaction(tcode.clone()) {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: format!("Couldn't execute transaction. {e}"),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }

            None
        })
    .with_instruction(
        Instruction::new(
            "sap-screenshot",
            "Screenshot as Evidence",
            "Take a screenshot of a SAP window"
        )
        .with_parameter("label", "Evidence Label", ParameterKind::String)
        .with_parameter("target", "Target (usually 'wnd[0]')", ParameterKind::String),
        |state, params, _output, evidence| {
            let mut state = state.lock().expect("state must be lockable");
            let label = params["label"].value_string();
            let target = params["target"].value_string();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Ok(wnd) = session.find_by_id(target.clone()) {
                        match match wnd {
                            SAPComponent::GuiMainWindow(wnd) => wnd
                                .hard_copy("evidence.png".to_string(), 2)
                                .map_err(|e| format!("Can't screenshot: {e}")),
                            SAPComponent::GuiFrameWindow(wnd) => wnd
                                .hard_copy("evidence.png".to_string(), 2)
                                .map_err(|e| format!("Can't screenshot: {e}")),
                            _ => Err("No valid target to screenshot.".to_string()),
                        } {
                            Ok(path) => {
                                // Read path, add to evidence, delete file
                                match fs::read(&path) {
                                    Ok(data) => {
                                        use base64::{Engine as _, engine::general_purpose};
                                        let b64_data = general_purpose::STANDARD.encode(&data);
                                        evidence.push(Evidence { label, content: EvidenceContent::ImageAsPngBase64(b64_data) });

                                        // try to delete, but don't worry if we can't
                                        let _ = fs::remove_file(path);
                                    },
                                    Err(e) => {
                                        return Some(Response::Error {
                                            kind: ErrorKind::EngineProcessingError,
                                            reason: format!("Failed to read screenshot: {e}"),
                                        })
                                    }
                                }
                            }
                            Err(reason) => return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason,
                            })
                        }
                    } else {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: format!("Couldn't find {target}."),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }

            None
        }
    )
    .with_instruction(
        Instruction::new(
            "sap-set-text-value",
            "Text Value: Set",
            "Set the value of a fields 'Text' value. The behaviour of this differs depending on the type of field.",
        )
        .with_parameter("target", "Target", ParameterKind::String)
        .with_parameter("value", "Value", ParameterKind::String),
        |state, params, _output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");
            let target = params["target"].value_string();
            let value = params["value"].value_string();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Ok(wnd) = session.find_by_id(target.clone()) {
                        if let Err(reason) = match wnd {
                            SAPComponent::GuiTextField(txt) => txt
                                .set_text(value)
                                .map_err(|e| format!("Can't set text: {e}")),
                            SAPComponent::GuiCTextField(txt) => txt
                                .set_text(value)
                                .map_err(|e| format!("Can't set text: {e}")),
                            _ => Err("No valid target to set text.".to_string()),
                        } {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason,
                            })
                        }
                    } else {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: format!("Couldn't find {target}."),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }

            None
        })
    .with_instruction(
        Instruction::new(
            "sap-get-text-value",
            "Text Value: Get",
            "Get the value of a fields 'Text' value. The behaviour of this differs depending on the type of field.",
        )
        .with_parameter("target", "Target", ParameterKind::String)
        .with_output("value", "Value", ParameterKind::String),
        |state, params, output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");
            let target = params["target"].value_string();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Ok(wnd) = session.find_by_id(target.clone()) {
                        match match wnd {
                            SAPComponent::GuiTextField(txt) => {
                                txt.text().map_err(|e| format!("Can't get text: {e}"))
                            }
                            SAPComponent::GuiCTextField(txt) => {
                                txt.text().map_err(|e| format!("Can't get text: {e}"))
                            }
                            SAPComponent::GuiFrameWindow(txt) => {
                                txt.text().map_err(|e| format!("Can't get text: {e}"))
                            }
                            SAPComponent::GuiMainWindow(txt) => {
                                txt.text().map_err(|e| format!("Can't get text: {e}"))
                            }
                            _ => Err("No valid target to get text.".to_string()),
                        } {
                            Ok(text) => {
                                output.insert("value".to_string(), ParameterValue::String(text));
                            }
                            Err(reason) => {
                                return Some(Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason,
                                })
                            }
                        }
                    } else {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: format!("Couldn't find {target}."),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }
            None
        })
    .with_instruction(
        Instruction::new(
            "sap-send-key",
            "Send Key",
            "Send a keypress to the SAP system.",
        )
        .with_parameter("key", "Key (VKey)", ParameterKind::Integer),
        |state, params, _output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");

            let key = params["key"].value_i32();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Ok(wnd) = session.find_by_id("wnd[0]".to_owned()) {
                        if let Err(reason) = match wnd {
                            SAPComponent::GuiMainWindow(wnd) => wnd
                                .send_v_key(key)
                                .map_err(|e| format!("Couldn't send VKey: {e}")),
                            _ => Err(String::from("SAP window not open")),
                        } {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason,
                            })
                        }
                    } else {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: String::from("SAP window couldn't be requested."),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }

            None
        })
        .with_instruction(
            Instruction::new(
                "sap-press-button",
                "Press UI Button",
                "Press a button in the UI.",
            )
            .with_parameter("target", "Target", ParameterKind::String),
            |state, params, _output, _evidence| {
                let mut state = state.lock().expect("state must be lockable");
                let id = params["target"].value_string();

                match get_session(&mut *state) {
                    Ok(session) => {
                        if let Ok(comp) = session.find_by_id(id) {
                            if let Err(reason) = match comp {
                                SAPComponent::GuiButton(b) => {
                                    b.press().map_err(|e| format!("Couldn't press button: {e}"))
                                }
                                _ => Err(String::from("Tried to press a non-button")),
                            } {
                                return Some(Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason,
                                })
                            }
                        } else {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: String::from("Failed to find component"),
                            })
                        }
                    }
                    Err(e) => {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: e,
                        })
                    }
                }
                None
            })
            .with_instruction(
                Instruction::new(
                    "sap-set-checkbox",
                    "Checkbox: Set Value",
                    "Set the state of a checkbox in the UI.",
                )
                .with_parameter("target", "Target", ParameterKind::String)
                .with_parameter("state", "Checked", ParameterKind::Boolean),
                |state, params, _output, _evidence| {
                    let mut state = state.lock().expect("state must be lockable");
                    let id = params["target"].value_string();
                    let cb_state = params["state"].value_bool();

                    match get_session(&mut *state) {
                        Ok(session) => {
                            if let Ok(comp) = session.find_by_id(id) {
                                if let Err(reason) = match comp {
                                    SAPComponent::GuiCheckBox(c) => c
                                        .set_selected(cb_state)
                                        .map_err(|e| format!("Couldn't set checkbox: {e}")),
                                    _ => Err(String::from("Tried to check a non-checkbox")),
                                } {
                                    return Some(Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    })
                                }
                            } else {
                                return Some(Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: String::from("Failed to find component"),
                                })
                            }
                        }
                        Err(e) => {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            })
                        }
                    }
                    None
                })
        .with_instruction(
            Instruction::new(
                "sap-set-combobox-key",
                "Combo Box: Set Key",
                "Set the key (selected item) of the combo box.",
            )
            .with_parameter("target", "Target", ParameterKind::String)
            .with_parameter("key", "Key", ParameterKind::String),
            |state, params, _output, _evidence| {
                let mut state = state.lock().expect("state must be lockable");
                let target = params["target"].value_string();
                let key = params["key"].value_string();

                match get_session(&mut *state) {
                    Ok(session) => {
                        if let Ok(wnd) = session.find_by_id(target.clone()) {
                            if let Err(reason) = match wnd {
                                SAPComponent::GuiComboBox(cmb) => cmb
                                    .set_key(key)
                                    .map_err(|e| format!("Can't set combo box key: {e}")),
                                _ => Err("No valid target to set combo box key.".to_string()),
                            } {
                                return Some(Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason,
                                })
                            }
                        } else {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: format!("Couldn't find {target}."),
                            })
                        }
                    }
                    Err(e) => {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: e,
                        })
                    }
                }
                None
            })
    .with_instruction(
        Instruction::new(
            "sap-grid-get-row-count",
            "Grid: Get Row Count",
            "Get the number of rows in a grid.",
        )
        .with_parameter("target", "Target Grid", ParameterKind::String)
        .with_output("value", "Number of rows", ParameterKind::Integer),
        |state, params, output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");
            let id = params["target"].value_string();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Ok(comp) = session.find_by_id(id) {
                        if let Err(reason) = match comp {
                            SAPComponent::GuiGridView(g) => {
                                if let Ok(row_count) = g.row_count() {
                                    // ! This might drop some precision in some situations!
                                    output.insert(
                                        "value".to_string(),
                                        ParameterValue::Integer(row_count as i32),
                                    );
                                    Ok(())
                                } else {
                                    Err(String::from("The grid had no row count."))
                                }
                            }
                            _ => Err(String::from("The grid was invalid")),
                        } {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason,
                            })
                        }
                    } else {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: String::from("Failed to find grid"),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }

            None
        })
    .with_instruction(
        Instruction::new(
            "sap-grid-click-cell",
            "Grid: Click or Double Click Cell",
            "Click or double click a cell.",
        )
        .with_parameter("target", "Target Grid", ParameterKind::String)
        .with_parameter("row", "Row", ParameterKind::Integer)
        .with_parameter("col", "Column", ParameterKind::String)
        .with_parameter("double", "Double click", ParameterKind::Boolean),
        |state, params, _output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");
            let id = params["target"].value_string();
            let row = params["row"].value_i32();
            let col = params["col"].value_string();
            let double = params["double"].value_bool();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Ok(comp) = session.find_by_id(id) {
                        if let Err(reason) = match comp {
                            SAPComponent::GuiGridView(g) => {
                                if double {
                                    g.double_click(row as i64, col).map_err(|_| {
                                        String::from("The grid couldn't be double clicked.")
                                    })
                                } else {
                                    g.click(row as i64, col).map_err(|_| {
                                        String::from("The grid couldn't be clicked.")
                                    })
                                }
                            }
                            _ => Err(String::from("The grid was invalid")),
                        } {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason,
                            })
                        }
                    } else {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: String::from("Failed to find grid"),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }
            None
        })
    .with_instruction(
        Instruction::new(
            "sap-grid-get-cell-value",
            "Grid: Get Cell Value",
            "Get the value of a grid cell.",
        )
        .with_parameter("target", "Target Grid", ParameterKind::String)
        .with_parameter("row", "Row", ParameterKind::Integer)
        .with_parameter("col", "Column", ParameterKind::String)
        .with_output("value", "Value", ParameterKind::String),
        |state, params, output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");
            let id = params["target"].value_string();
            let row = params["row"].value_i32();
            let col = params["col"].value_string();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Ok(comp) = session.find_by_id(id) {
                        if let Err(reason) = match comp {
                            SAPComponent::GuiGridView(g) => {
                                if let Ok(value) = g.get_cell_value(row as i64, col) {
                                    output.insert(
                                        "value".to_string(),
                                        ParameterValue::String(value),
                                    );
                                    Ok(())
                                } else {
                                    Err(String::from("The statusbar had no message type."))
                                }
                            }
                            _ => Err(String::from("The statusbar was invalid")),
                        } {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason,
                            })
                        }
                    } else {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: String::from("Failed to find status bar"),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }

            None
        })
    .with_instruction(
        Instruction::new(
            "sap-get-statusbar-state",
            "Status Bar: Get State",
            "Get the type of message displayed in the status bar shown at the bottom of the SAP window. This could be 'S' (Success), 'W' (Warning), 'E' (Error), 'A' (Abort), 'I' (Information) or '' (No Status).",
        )
        .with_parameter("target", "Target (usually 'wnd[0]/sbar')", ParameterKind::String)
        .with_output("status", "Status", ParameterKind::String),
        |state, params, output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");
            let id = params["target"].value_string();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Ok(comp) = session.find_by_id(id) {
                        if let Err(reason) = match comp {
                            SAPComponent::GuiStatusbar(s) => {
                                if let Ok(status) = s.message_type() {
                                    output.insert(
                                        "status".to_string(),
                                        ParameterValue::String(status),
                                    );
                                    Ok(())
                                } else {
                                    Err(String::from("The statusbar had no message type."))
                                }
                            }
                            _ => Err(String::from("The statusbar was invalid")),
                        } {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason,
                            })
                        }
                    } else {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: String::from("Failed to find status bar"),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }

            None
        })
    .with_instruction(
        Instruction::new(
            "sap-tab-select",
            "Tab: Select",
            "Select a tab in a tab panel.",
        )
        .with_parameter("target", "Target Tab", ParameterKind::String),
        |state, params, _output, _evidence| {
            let mut state = state.lock().expect("state must be lockable");
            let id = params["target"].value_string();

            match get_session(&mut *state) {
                Ok(session) => {
                    if let Ok(comp) = session.find_by_id(id) {
                        if let Err(reason) = match comp {
                            SAPComponent::GuiTab(g) => g.select().map_err(|_| {
                                String::from("The tab couldn't be selected.")
                            }),
                            _ => Err(String::from("The tab was invalid")),
                        } {
                            return Some(Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason,
                            })
                        }
                    } else {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: String::from("Failed to find tab"),
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: e,
                    })
                }
            }
            None
        })
    );
}

expose_engine!(ENGINE);

fn connect(state: &mut State) -> std::result::Result<(), String> {
    let com_instance = SAPComInstance::new().map_err(|_| "Couldn't get COM instance")?;
    let wrapper = com_instance
        .sap_wrapper()
        .map_err(|e| format!("Couldn't get SAP wrapper: {e}"))?;
    let engine = wrapper
        .scripting_engine()
        .map_err(|e| format!("Couldn't get GuiApplication instance: {e}"))?;

    let connection = match sap_scripting::GuiApplication_Impl::children(&engine)
        .map_err(|e| format!("Couldn't get GuiApplication children: {e}"))?
        .element_at(0)
        .map_err(|e| format!("Couldn't get child of GuiApplication: {e}"))?
    {
        SAPComponent::GuiConnection(conn) => conn,
        _ => {
            return Err(String::from(
                "Expected GuiConnection, but got something else!",
            ))
        }
    };
    let session = match sap_scripting::GuiConnection_Impl::children(&connection)
        .map_err(|e| format!("Couldn't get GuiConnection children: {e}"))?
        .element_at(0)
        .map_err(|e| format!("Couldn't get child of GuiConnection: {e}"))?
    {
        SAPComponent::GuiSession(session) => session,
        _ => return Err(String::from("Expected GuiSession, but got something else!")),
    };

    state.com_instance = Some(com_instance);
    state.session = Some(session);

    Ok(())
}

fn get_session(state: &State) -> std::result::Result<&GuiSession, String> {
    state
        .session
        .as_ref()
        .ok_or("GuiSession not initialised".to_string())
}
