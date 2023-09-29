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
    static ref ENGINE: Mutex<Engine<'static, Mutex<State>>> = Mutex::new(Engine::new("SAP", env!("CARGO_PKG_VERSION"))
    .with_instruction(
        Instruction::new(
            "sap-connect",
            "Connect to Open Instance",
            "Connect to an SAP instance that the user already has open.\nIf they have multiple open, this will give access to any of the open windows (although most instructions use the main window).\nThis will do nothing if we already hold a connection.",
        ),
        |state: &mut Mutex<State>, _params, _output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");

            if state.com_instance.is_none() {
                connect(state)?;
            }

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-run-transaction",
            "Run Transaction",
            "Run a transaction.",
        )
        .with_parameter("tcode", "Transaction Code", ParameterKind::String),
        |state, params, _output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let tcode = params["tcode"].value_string();

            let session = get_session(state)?;
            session.start_transaction(tcode.clone()).map_err(|e| format!("Couldn't execute transaction. {e}"))?;

            Ok(())
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
            let state = state.get_mut().expect("state must be lockable");
            let label = params["label"].value_string();
            let target = params["target"].value_string();

            let session = get_session(state)?;
            let wnd = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            let path = match wnd {
                SAPComponent::GuiMainWindow(wnd) => wnd
                    .hard_copy("evidence.png".to_string(), 2)
                    .map_err(|e| format!("Can't screenshot: {e}")),
                SAPComponent::GuiFrameWindow(wnd) => wnd
                    .hard_copy("evidence.png".to_string(), 2)
                    .map_err(|e| format!("Can't screenshot: {e}")),
                _ => Err("No valid target to screenshot.".to_string()),
            }?;
            // Read path, add to evidence, delete file
            let data = fs::read(&path).map_err(|e| format!("Failed to read screenshot: {e}"))?;

            use base64::{Engine as _, engine::general_purpose};
            let b64_data = general_purpose::STANDARD.encode(data);
            evidence.push(Evidence { label, content: EvidenceContent::ImageAsPngBase64(b64_data) });

            // try to delete, but don't worry if we can't
            let _ = fs::remove_file(path);

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-does-element-exist",
            "Does Element Exist",
            "Check if an element exists and returns a boolean.",
        )
        .with_parameter("target", "Target", ParameterKind::String)
        .with_output("exists", "Exists", ParameterKind::Boolean),
        |state, params, output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let target = params["target"].value_string();

            let session = get_session(state)?;
            output.insert("exists".to_string(), ParameterValue::Boolean(session.find_by_id(target.clone()).is_ok()));

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-component-type",
            "Component Type",
            "Return the type string of the component."
        )
        .with_parameter("target", "Target", ParameterKind::String)
        .with_output("type", "Type", ParameterKind::String),
        |state, params, output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let target = params["target"].value_string();

            let session = get_session(state)?;
            let comp = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            let kind = match comp {
                SAPComponent::GuiBarChart(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiBox(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiButton(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiCalendar(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiChart(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiCheckBox(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiColorSelector(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiComboBox(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiComboBoxControl(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiContainerShell(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiCTextField(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiCustomControl(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiDialogShell(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiEAIViewer2D(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiEAIViewer3D(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiFrameWindow(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiGOSShell(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiGraphAdapt(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiGridView(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiHTMLViewer(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiInputFieldControl(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiLabel(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiMainWindow(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiMap(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiMenu(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiMenubar(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiModalWindow(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiNetChart(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiOfficeIntegration(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiOkCodeField(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiPasswordField(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiPicture(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiRadioButton(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiSapChart(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiScrollContainer(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiShell(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiSimpleContainer(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiSplit(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiSplitterContainer(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiStage(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiStatusbar(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiStatusPane(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiTab(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiTableControl(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiTabStrip(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiTextedit(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiTextField(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiTitlebar(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiToolbar(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiTree(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiUserArea(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiVComponent(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiVContainer(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                SAPComponent::GuiVHViewSwitch(comp) => comp._type().map_err(|e| format!("Failed to get type: {e}")),
                _ => Err("No valid target to get type.".to_string()),
            }?;
            output.insert("type".to_string(), ParameterValue::String(kind));

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-visualise-element",
            "Highlight Element",
            "Highlight an element by drawing a red box around it. Useful just before screenshotting."
        )
        .with_parameter("target", "Target", ParameterKind::String),
        |state, params, _output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let target = params["target"].value_string();

            let session = get_session(state)?;
            let comp = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            match comp {
                SAPComponent::GuiBarChart(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiBox(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiButton(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiCalendar(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiChart(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiCheckBox(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiColorSelector(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiComboBox(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiComboBoxControl(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiContainerShell(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiCTextField(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiCustomControl(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiDialogShell(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiEAIViewer2D(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiEAIViewer3D(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiFrameWindow(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiGOSShell(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiGraphAdapt(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiGridView(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiHTMLViewer(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiInputFieldControl(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiLabel(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiMainWindow(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiMap(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiMenu(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiMenubar(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiModalWindow(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiNetChart(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiOfficeIntegration(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiOkCodeField(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiPasswordField(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiPicture(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiRadioButton(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiSapChart(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiScrollContainer(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiShell(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiSimpleContainer(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiSplit(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiSplitterContainer(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiStage(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiStatusbar(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiStatusPane(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiTab(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiTableControl(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiTabStrip(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiTextedit(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiTextField(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiTitlebar(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiToolbar(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiTree(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiUserArea(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiVComponent(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiVContainer(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                SAPComponent::GuiVHViewSwitch(comp) => comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}")),
                _ => Err("No valid target to visualise.".to_string()),
            }?;

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-set-text-value",
            "Text Value: Set",
            "Set the value of a fields 'Text' value. The behaviour of this differs depending on the type of field.",
        )
        .with_parameter("target", "Target", ParameterKind::String)
        .with_parameter("value", "Value", ParameterKind::String),
        |state, params, _output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let target = params["target"].value_string();
            let value = params["value"].value_string();

            let session = get_session(state)?;
            let wnd = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            match wnd {
                SAPComponent::GuiTextField(txt) => txt
                    .set_text(value)
                    .map_err(|e| format!("Can't set text: {e}")),
                SAPComponent::GuiCTextField(txt) => txt
                    .set_text(value)
                    .map_err(|e| format!("Can't set text: {e}")),
                _ => Err("No valid target to set text.".to_string()),
            }?;

            Ok(())
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
            let state = state.get_mut().expect("state must be lockable");
            let target = params["target"].value_string();

            let session = get_session(state)?;
            let wnd = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            let text = match wnd {
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
            }?;
            output.insert("value".to_string(), ParameterValue::String(text));

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-send-key",
            "Send Key",
            "Send a keypress to the SAP system.",
        )
        .with_parameter("key", "Key (VKey)", ParameterKind::Integer),
        |state, params, _output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");

            let key = params["key"].value_i32();

            let session = get_session(state)?;
            let wnd = session.find_by_id("wnd[0]".to_owned()).map_err(|_| String::from("SAP window couldn't be requested."))?;
            match wnd {
                SAPComponent::GuiMainWindow(wnd) => wnd
                    .send_v_key(key as i16)
                    .map_err(|e| format!("Couldn't send VKey: {e}")),
                _ => Err(String::from("SAP window not open")),
            }?;

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-press-button",
            "Press UI Button",
            "Press a button in the UI.",
        )
        .with_parameter("target", "Target", ParameterKind::String),
        |state, params, _output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let id = params["target"].value_string();

            let session = get_session(state)?;
            let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find component"))?;
            match comp {
                SAPComponent::GuiButton(b) => {
                    b.press().map_err(|e| format!("Couldn't press button: {e}"))
                }
                _ => Err(String::from("Tried to press a non-button")),
            }?;

            Ok(())
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
                let state = state.get_mut().expect("state must be lockable");
                let id = params["target"].value_string();
                let cb_state = params["state"].value_bool();

                let session = get_session(state)?;
                let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find component"))?;
                match comp {
                    SAPComponent::GuiCheckBox(c) => c
                        .set_selected(cb_state)
                        .map_err(|e| format!("Couldn't set checkbox: {e}")),
                    _ => Err(String::from("Tried to check a non-checkbox")),
                }?;

                Ok(())
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
            let state = state.get_mut().expect("state must be lockable");
            let target = params["target"].value_string();
            let key = params["key"].value_string();

            let session = get_session(state)?;
            let wnd = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            match wnd {
                SAPComponent::GuiComboBox(cmb) => cmb
                    .set_key(key)
                    .map_err(|e| format!("Can't set combo box key: {e}")),
                _ => Err("No valid target to set combo box key.".to_string()),
            }?;

            Ok(())
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
            let state = state.get_mut().expect("state must be lockable");
            let id = params["target"].value_string();

            let session = get_session(state)?;
            let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find grid"))?;
            match comp {
                SAPComponent::GuiGridView(g) => {
                    if let Ok(row_count) = g.row_count() {
                        // ! This might drop some precision in some situations!
                        output.insert(
                            "value".to_string(),
                            ParameterValue::Integer(row_count),
                        );
                        Ok(())
                    } else {
                        Err(String::from("The grid had no row count."))
                    }
                }
                _ => Err(String::from("The grid was invalid")),
            }?;

            Ok(())
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
            let state = state.get_mut().expect("state must be lockable");
            let id = params["target"].value_string();
            let row = params["row"].value_i32();
            let col = params["col"].value_string();
            let double = params["double"].value_bool();

            let session = get_session(state)?;
            let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find grid"))?;
            match comp {
                SAPComponent::GuiGridView(g) => {
                    g.set_current_cell(row, col).map_err(|e| format!("Couldn't select cell in grid: {e}"))?;
                    if double {
                        g.double_click_current_cell().map_err(|e| {
                            format!("The grid couldn't be double clicked: {e}")
                        })?;
                    } else {
                        g.click_current_cell().map_err(|e| {
                            format!("The grid couldn't be clicked: {e}")
                        })?;
                    }
                    Ok(())
                }
                _ => Err(String::from("The grid was invalid.")),
            }?;

            Ok(())
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
            let state = state.get_mut().expect("state must be lockable");
            let id = params["target"].value_string();
            let row = params["row"].value_i32();
            let col = params["col"].value_string();

            let session = get_session(state)?;
            let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find grid view"))?;
            match comp {
                SAPComponent::GuiGridView(g) => {
                    match g.get_cell_value(row, col) {
                        Ok(value) => {
                            output.insert(
                                "value".to_string(),
                                ParameterValue::String(value),
                            );
                            Ok(())
                        }
                        Err(e) => Err(format!("The value couldn't be read: {e}")),
                    }
                }
                _ => Err(String::from("The grid view was invalid")),
            }?;

            Ok(())
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
            let state = state.get_mut().expect("state must be lockable");
            let id = params["target"].value_string();

            let session = get_session(state)?;
            let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find status bar"))?;
            match comp {
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
            }?;

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-tab-select",
            "Tab: Select",
            "Select a tab in a tab panel.",
        )
        .with_parameter("target", "Target Tab", ParameterKind::String),
        |state, params, _output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let id = params["target"].value_string();

            let session = get_session(state)?;
            let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find tab"))?;
            match comp {
                SAPComponent::GuiTab(g) => g.select().map_err(|_| {
                    String::from("The tab couldn't be selected.")
                }),
                _ => Err(String::from("The tab was invalid")),
            }?;

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-table-get-row-count",
            "Table: Get Row Count",
            "Get the number of rows in a table.",
        )
        .with_parameter("target", "Target", ParameterKind::String)
        .with_output("rows", "Rows", ParameterKind::Integer),
        |state, params, output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let id = params["target"].value_string();

            let session = get_session(state)?;
            let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find table"))?;
            let num_rows = match comp {
                SAPComponent::GuiTableControl(tab) => tab.row_count().map_err(|e| format!("Cannot read number of rows: {e}")),
                _ => Err(String::from("The table was invalid")),
            }?;
            output.insert("rows".to_string(), ParameterValue::Integer(num_rows));

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-table-row-select",
            "Table: Row: Select",
            "Select a row in a table.",
        )
        .with_parameter("target", "Target", ParameterKind::String)
        .with_parameter("row", "Row", ParameterKind::Integer),
        |state, params, _output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let id = params["target"].value_string();
            let row = params["row"].value_i32();

            let session = get_session(state)?;
            let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find tab"))?;
            let row_comp = match comp {
                SAPComponent::GuiTableControl(tab) => tab.get_absolute_row(row).map_err(|e| format!("Failed to get table row: {e}")),
                _ => Err(String::from("The table was invalid")),
            }?;
            row_comp.set_selected(true).map_err(|e| format!("Failed to select row: {e}"))?;

            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "sap-table-cell-get-id",
            "Table: Get ID of Cell",
            "Get the ID of a cell that can be fed into another function's 'Target' parameter.",
        )
        .with_parameter("target", "Target", ParameterKind::String)
        .with_parameter("row", "Row", ParameterKind::Integer)
        .with_parameter("column", "Column", ParameterKind::Integer)
        .with_output("id", "Target ID", ParameterKind::String),
        |state, params, output, _evidence| {
            let state = state.get_mut().expect("state must be lockable");
            let id = params["target"].value_string();
            let row = params["row"].value_i32();
            let col = params["column"].value_i32();

            let session = get_session(state)?;
            let comp = session.find_by_id(id).map_err(|_| String::from("Failed to find tab"))?;
            let comp = match comp {
                SAPComponent::GuiTableControl(tab) => tab.get_cell(row, col).map_err(|e| format!("Failed to get table cell: {e}")),
                _ => Err(String::from("The table was invalid")),
            }?;
            let id = match comp {
                SAPComponent::GuiApplication(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiBarChart(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiBox(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiButton(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiCalendar(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiChart(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiCheckBox(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiColorSelector(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiComboBox(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiComboBoxControl(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiComponent(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiContainer(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiContainerShell(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiCTextField(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiCustomControl(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiDialogShell(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiEAIViewer2D(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiEAIViewer3D(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiFrameWindow(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiGOSShell(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiGraphAdapt(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiGridView(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiHTMLViewer(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiInputFieldControl(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiLabel(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiMainWindow(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiMap(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiMenu(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiMenubar(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiModalWindow(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiNetChart(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiOfficeIntegration(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiOkCodeField(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiPasswordField(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiPicture(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiRadioButton(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiSapChart(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiScrollContainer(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiShell(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiSimpleContainer(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiSplit(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiSplitterContainer(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiStage(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiStatusbar(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiStatusPane(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiTab(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiTableControl(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiTabStrip(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiTextedit(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiTextField(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiTitlebar(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiToolbar(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiTree(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiUserArea(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiVComponent(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiVContainer(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                SAPComponent::GuiVHViewSwitch(comp) => comp.id().map_err(|e| format!("Failed to get ID: {e}")),
                _ => Err("Invalid component to get ID.".to_string()),
            }?;
            output.insert("id".to_string(), ParameterValue::String(id));

            Ok(())
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
