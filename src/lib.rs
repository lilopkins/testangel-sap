use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
    sync::Mutex,
};

use lazy_static::lazy_static;
use sap_scripting::*;
use testangel_ipc::prelude::*;

#[derive(Default)]
struct State {
    com_instance: Option<SAPComInstance>,
    session: Option<GuiSession>,
}

unsafe impl Send for State {}

lazy_static! {
    static ref INSTRUCTION_CONNECT_TO_OPEN_INSTANCE: Instruction = Instruction::new(
        "sap-connect",
        "Connect to Open Instance",
        "Connect to an SAP instance that the user already has open.\nIf they have multiple open, this will give access to any of the open windows (although most instructions use the main window).\nThis will do nothing if we already hold a connection.",
    );
    static ref INSTRUCTION_RUN_TRANSACTION: Instruction = Instruction::new(
        "sap-run-transaction",
        "Run Transaction",
        "Run a transaction.",
    )
    .with_parameter("tcode", "Transaction Code", ParameterKind::String);
    static ref INSTRUCTION_SET_TEXT_VALUE: Instruction = Instruction::new(
        "sap-set-text-value",
        "Set Text Value",
        "Set the value of a fields 'Text' value. The behaviour of this differs depending on the type of field.",
    )
    .with_parameter("target", "Target", ParameterKind::String)
    .with_parameter("value", "Value", ParameterKind::String);
    static ref INSTRUCTION_GET_TEXT_VALUE: Instruction = Instruction::new(
        "sap-get-text-value",
        "Get Text Value",
        "Get the value of a fields 'Text' value. The behaviour of this differs depending on the type of field.",
    )
    .with_parameter("target", "Target", ParameterKind::String)
    .with_output("value", "Value", ParameterKind::String);
    static ref INSTRUCTION_SET_COMBOBOX_KEY: Instruction = Instruction::new(
        "sap-set-combobox-key",
        "Set Combo box Key",
        "Set the key (selected item) of the combo box.",
    )
    .with_parameter("target", "Target", ParameterKind::String)
    .with_parameter("key", "Key", ParameterKind::String);
    static ref INSTRUCTION_SEND_KEY: Instruction = Instruction::new(
        "sap-send-key",
        "Send Key",
        "Send a keypress to the SAP system.",
    )
    .with_parameter("key", "Key (VKey)", ParameterKind::Integer);
    static ref INSTRUCTION_PRESS_BUTTON: Instruction = Instruction::new(
        "sap-press-button",
        "Press Button",
        "Press a button in the UI.",
    )
    .with_parameter("target", "Target", ParameterKind::String);
    static ref INSTRUCTION_SET_CHECKBOX: Instruction = Instruction::new(
        "sap-set-checkbox",
        "Set Checkbox",
        "Set the state of a checkbox in the UI.",
    )
    .with_parameter("target", "Target", ParameterKind::String)
    .with_parameter("state", "Checked", ParameterKind::Boolean);
    static ref INSTRUCTION_GET_STATUSBAR_STATE: Instruction = Instruction::new(
        "sap-get-statusbar-state",
        "Get Status Bar State",
        "Get the type of message displayed in the status bar shown at the bottom of the SAP window. This could be 'S' (Success), 'W' (Warning), 'E' (Error), 'A' (Abort), 'I' (Information) or '' (No Status).",
    )
    .with_parameter("target", "Target (usually 'wnd[0]/sbar')", ParameterKind::String)
    .with_output("status", "Status", ParameterKind::String);
    static ref INSTRUCTION_GRID_GET_ROW_COUNT: Instruction = Instruction::new(
        "sap-grid-get-row-count",
        "Grid: Get Row Count",
        "Get the number of rows in a grid.",
    )
    .with_parameter("target", "Target Grid", ParameterKind::String)
    .with_output("value", "Number of rows", ParameterKind::Integer);
    static ref INSTRUCTION_GRID_CLICK_CELL: Instruction = Instruction::new(
        "sap-grid-click-cell",
        "Grid: Click or Double Click Cell",
        "Click or double click a cell.",
    )
    .with_parameter("target", "Target Grid", ParameterKind::String)
    .with_parameter("row", "Row", ParameterKind::Integer)
    .with_parameter("col", "Column", ParameterKind::String)
    .with_parameter("double", "Double click", ParameterKind::Boolean);
    static ref INSTRUCTION_GRID_GET_CELL_VALUE: Instruction = Instruction::new(
        "sap-grid-get-cell-value",
        "Grid: Get Cell Value",
        "Get the value of a grid cell.",
    )
    .with_parameter("target", "Target Grid", ParameterKind::String)
    .with_parameter("row", "Row", ParameterKind::Integer)
    .with_parameter("col", "Column", ParameterKind::String)
    .with_output("value", "Value", ParameterKind::String);
    static ref INSTRUCTION_TAB_SELECT: Instruction = Instruction::new(
        "sap-tab-select",
        "Select Tab",
        "Select a tab in a tab panel.",
    )
    .with_parameter("target", "Target Tab", ParameterKind::String);
    static ref STATE: Mutex<State> = Mutex::new(State::default());
}

#[no_mangle]
pub unsafe extern "C" fn ta_call(input: *const c_char) -> *mut c_char {
    let input = CStr::from_ptr(input);
    let response = call_internal(String::from_utf8_lossy(input.to_bytes()).to_string());
    let c_response = CString::new(response).expect("valid response");
    c_response.into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn ta_release(input: *mut c_char) {
    if !input.is_null() {
        drop(CString::from_raw(input));
    }
}

fn call_internal(request_str: String) -> String {
    // Parse the request
    let request = Request::try_from(request_str);
    if let Err(e) = request {
        // Return a well-formatted error if the request couldn't be parsed.
        return Response::Error {
            kind: ErrorKind::FailedToParseIPCJson,
            reason: format!("The IPC message was invalid. ({:?})", e),
        }
        .to_json();
    }
    let request = request.unwrap();
    let res = process_request(STATE.lock().as_deref_mut().unwrap(), request);
    res.to_json()
}

fn process_request(state: &mut State, request: Request) -> Response {
    match request {
        Request::ResetState => {
            // Reset the state.
            *state = State::default();
            Response::StateReset
        }
        Request::Instructions => {
            // Provide a list of instructions this engine can run.
            Response::Instructions {
                friendly_name: "SAP".to_owned(),
                instructions: vec![
                    INSTRUCTION_CONNECT_TO_OPEN_INSTANCE.clone(),
                    INSTRUCTION_RUN_TRANSACTION.clone(),
                    INSTRUCTION_SET_TEXT_VALUE.clone(),
                    INSTRUCTION_GET_TEXT_VALUE.clone(),
                    INSTRUCTION_SET_COMBOBOX_KEY.clone(),
                    INSTRUCTION_SEND_KEY.clone(),
                    INSTRUCTION_PRESS_BUTTON.clone(),
                    INSTRUCTION_SET_CHECKBOX.clone(),
                    INSTRUCTION_GET_STATUSBAR_STATE.clone(),
                    INSTRUCTION_GRID_GET_ROW_COUNT.clone(),
                    INSTRUCTION_GRID_CLICK_CELL.clone(),
                    INSTRUCTION_GRID_GET_CELL_VALUE.clone(),
                ],
            }
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_CONNECT_TO_OPEN_INSTANCE.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_CONNECT_TO_OPEN_INSTANCE.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    if state.com_instance.is_none() {
                        match connect(state) {
                            Err(e) => {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: e,
                                }
                            }
                            Ok(_) => (),
                        }
                    }

                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_RUN_TRANSACTION.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_RUN_TRANSACTION.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let tcode = i.parameters["tcode"].value_string();

                    match get_session(state) {
                        Ok(session) => {
                            if let Err(e) = session.start_transaction(tcode.clone()) {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: format!("Couldn't execute transaction. {e}"),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![Evidence {
                        label: "SAP Transaction".to_owned(),
                        content: EvidenceContent::Textual(format!("Ran transaction '{tcode}'.")),
                    }]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_SET_TEXT_VALUE.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_SET_TEXT_VALUE.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let target = i.parameters["target"].value_string();
                    let value = i.parameters["value"].value_string();

                    match get_session(state) {
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
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: format!("Couldn't find {target}."),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_GET_TEXT_VALUE.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_GET_TEXT_VALUE.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let target = i.parameters["target"].value_string();
                    let mut o = HashMap::new();

                    match get_session(state) {
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
                                        o.insert("value".to_string(), ParameterValue::String(text));
                                    }
                                    Err(reason) => {
                                        return Response::Error {
                                            kind: ErrorKind::EngineProcessingError,
                                            reason,
                                        }
                                    }
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: format!("Couldn't find {target}."),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(o);
                } else if i.instruction == *INSTRUCTION_SET_COMBOBOX_KEY.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_SET_COMBOBOX_KEY.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let target = i.parameters["target"].value_string();
                    let key = i.parameters["key"].value_string();

                    match get_session(state) {
                        Ok(session) => {
                            if let Ok(wnd) = session.find_by_id(target.clone()) {
                                if let Err(reason) = match wnd {
                                    SAPComponent::GuiComboBox(cmb) => cmb
                                        .set_key(key)
                                        .map_err(|e| format!("Can't set combo box key: {e}")),
                                    _ => Err("No valid target to set combo box key.".to_string()),
                                } {
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: format!("Couldn't find {target}."),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_SEND_KEY.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_SEND_KEY.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let key = i.parameters["key"].value_i32();

                    match get_session(state) {
                        Ok(session) => {
                            if let Ok(wnd) = session.find_by_id("wnd[0]".to_owned()) {
                                if let Err(reason) = match wnd {
                                    SAPComponent::GuiMainWindow(wnd) => wnd
                                        .send_v_key(key)
                                        .map_err(|e| format!("Couldn't send VKey: {e}")),
                                    _ => Err(String::from("SAP window not open")),
                                } {
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: String::from("SAP window couldn't be requested."),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_PRESS_BUTTON.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_PRESS_BUTTON.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let id = i.parameters["target"].value_string();

                    match get_session(state) {
                        Ok(session) => {
                            if let Ok(comp) = session.find_by_id(id) {
                                if let Err(reason) = match comp {
                                    SAPComponent::GuiButton(b) => {
                                        b.press().map_err(|e| format!("Couldn't press button: {e}"))
                                    }
                                    _ => Err(String::from("Tried to press a non-button")),
                                } {
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: String::from("Failed to find component"),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_SET_CHECKBOX.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_SET_CHECKBOX.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let id = i.parameters["target"].value_string();
                    let cb_state = i.parameters["state"].value_bool();

                    match get_session(state) {
                        Ok(session) => {
                            if let Ok(comp) = session.find_by_id(id) {
                                if let Err(reason) = match comp {
                                    SAPComponent::GuiCheckBox(c) => c
                                        .set_selected(cb_state)
                                        .map_err(|e| format!("Couldn't set checkbox: {e}")),
                                    _ => Err(String::from("Tried to check a non-checkbox")),
                                } {
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: String::from("Failed to find component"),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_GET_STATUSBAR_STATE.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_GET_STATUSBAR_STATE.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let id = i.parameters["target"].value_string();
                    let mut o = HashMap::new();

                    match get_session(state) {
                        Ok(session) => {
                            if let Ok(comp) = session.find_by_id(id) {
                                if let Err(reason) = match comp {
                                    SAPComponent::GuiStatusbar(s) => {
                                        if let Ok(status) = s.message_type() {
                                            o.insert(
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
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: String::from("Failed to find status bar"),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(o);
                } else if i.instruction == *INSTRUCTION_GRID_GET_ROW_COUNT.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_GRID_GET_ROW_COUNT.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let id = i.parameters["target"].value_string();
                    let mut o = HashMap::new();

                    match get_session(state) {
                        Ok(session) => {
                            if let Ok(comp) = session.find_by_id(id) {
                                if let Err(reason) = match comp {
                                    SAPComponent::GuiGridView(g) => {
                                        if let Ok(row_count) = g.row_count() {
                                            // ! This might drop some precision in some situations!
                                            o.insert(
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
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: String::from("Failed to find grid"),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(o);
                } else if i.instruction == *INSTRUCTION_GRID_CLICK_CELL.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_GRID_CLICK_CELL.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let id = i.parameters["target"].value_string();
                    let row = i.parameters["row"].value_i32();
                    let col = i.parameters["col"].value_string();
                    let double = i.parameters["double"].value_bool();

                    match get_session(state) {
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
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: String::from("Failed to find grid"),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_GRID_GET_CELL_VALUE.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_GRID_GET_CELL_VALUE.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let id = i.parameters["target"].value_string();
                    let row = i.parameters["row"].value_i32();
                    let col = i.parameters["col"].value_string();
                    let mut o = HashMap::new();

                    match get_session(state) {
                        Ok(session) => {
                            if let Ok(comp) = session.find_by_id(id) {
                                if let Err(reason) = match comp {
                                    SAPComponent::GuiGridView(g) => {
                                        if let Ok(value) = g.get_cell_value(row as i64, col) {
                                            o.insert(
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
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: String::from("Failed to find status bar"),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(o);
                } else if i.instruction == *INSTRUCTION_TAB_SELECT.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_TAB_SELECT.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    let id = i.parameters["target"].value_string();

                    match get_session(state) {
                        Ok(session) => {
                            if let Ok(comp) = session.find_by_id(id) {
                                if let Err(reason) = match comp {
                                    SAPComponent::GuiTab(g) => g.select().map_err(|_| {
                                        String::from("The tab couldn't be selected.")
                                    }),
                                    _ => Err(String::from("The tab was invalid")),
                                } {
                                    return Response::Error {
                                        kind: ErrorKind::EngineProcessingError,
                                        reason,
                                    };
                                }
                            } else {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: String::from("Failed to find tab"),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: e,
                            }
                        }
                    }

                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else {
                    return Response::Error {
                        kind: ErrorKind::InvalidInstruction,
                        reason: format!(
                            "The requested instruction {} could not be handled by this engine.",
                            i.instruction
                        ),
                    };
                }
            }
            // Print output
            Response::ExecutionOutput { output, evidence }
        }
    }
}

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
