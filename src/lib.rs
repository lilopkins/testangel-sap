#![warn(clippy::pedantic)]

use std::fs;

use sap_scripting::{
    GuiButton_Impl, GuiCheckBox_Impl, GuiComboBox_Impl, GuiComponentCollection_Impl, GuiComponent_Impl, GuiContainer_Impl, GuiFrameWindow_Impl, GuiGridView_Impl, GuiSession, GuiSession_Impl, GuiShell_Impl, GuiStatusbar_Impl, GuiTab_Impl, GuiTableControl_Impl, GuiTableRow_Impl, GuiVComponent_Impl, SAPComInstance, SAPComponent
};
use testangel_engine::{engine, Evidence, EvidenceContent};

engine! {
    /// Work with SAP
    #[engine(
        name = "SAP",
        version = env!("CARGO_PKG_VERSION"),
        lua_name = "SAP",
    )]
    #[derive(Default)]
    #[allow(clippy::upper_case_acronyms)]
    struct SAP {
        com_instance: Option<SAPComInstance>,
        session: Option<GuiSession>,
    }

    impl SAP {
        /// Connect to an SAP instance that the user already has open. If they
        /// have multiple open, this will give access to any of the open windows
        /// (although most instructions use the main window). This will do
        /// nothing if we already hold a connection.
        #[instruction(
            id = "sap-connect",
            lua_name = "Connect",
            name = "Connect to Open Instance",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn connect() {
            if state.com_instance.is_none() && !dry_run {
                connect(state)?;
            }
        }

        /// Run a transaction.
        #[instruction(
            id = "sap-run-transaction",
            lua_name = "RunTransaction",
            name = "Run Transaction",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn run_transaction(
            #[arg(name = "Transaction Code")] tcode: String,
        ) {
            let session = get_session(state)?;
            session.start_transaction(tcode.clone()).map_err(|e| format!("Couldn't execute transaction. {e}"))?;
        }

        /// Take a screenshot of a SAP window
        #[instruction(
            id = "sap-screenshot",
            lua_name = "ScreenshotAsEvidence",
            name = "Screenshot as Evidence",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn screenshot(
            #[arg(name = "Evidence Label")] label: String,
            #[arg(name = "Target (usually 'wnd[0]')")] target: String,
        ) {
            use base64::{Engine as _, engine::general_purpose};

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

            let b64_data = general_purpose::STANDARD.encode(data);
            evidence.push(Evidence { label, content: EvidenceContent::ImageAsPngBase64(b64_data) });

            // try to delete, but don't worry if we can't
            let _ = fs::remove_file(path);
        }

        /// Check if an element exists and returns a boolean.
        #[instruction(
            id = "sap-does-element-exist",
            lua_name = "DoesElementExist",
            name = "Does Element Exist",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn does_element_exist(
            target: String,
        ) -> #[output(id = "exists", name = "Exists")] bool {
            let session = get_session(state)?;
            session.find_by_id(target.clone()).is_ok()
        }

        /// Return the type string of the component.
        #[instruction(
            id = "sap-component-type",
            lua_name = "ComponentType",
            name = "Component Type",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn component_type(
            target: String,
        ) -> #[output(id = "type", name = "Type")] String {
            let session = get_session(state)?;
            let comp = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            match comp {
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
            }?
        }

        /// Highlight an element by drawing a red box around it. Useful just before screenshotting.
        #[instruction(
            id = "sap-visualise-element",
            lua_name = "HighlightElement",
            name = "Highlight Element",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn visualise_element(
            target: String,
        ) {
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
        }

        /// Set the value of a fields 'Text' value. The behaviour of this differs depending on the type of field.
        #[instruction(
            id = "sap-set-text-value",
            lua_name = "SetTextValue",
            name = "Text Value: Set",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn set_text_value(
            target: String,
            value: String,
        ) {
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
        }

        /// Get the value of a fields 'Text' value. The behaviour of this differs depending on the type of field.
        #[instruction(
            id = "sap-get-text-value",
            lua_name = "GetTextValue",
            name = "Text Value: Get",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn get_text_value(
            target: String,
        ) -> #[output(id = "value", name = "Value")] String {
            let session = get_session(state)?;
            let wnd = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            match wnd {
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
            }?
        }

        /// Send a keypress to the SAP system.
        #[instruction(
            id = "sap-send-key",
            lua_name = "SendKey",
            name = "Send Key",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn send_key(
            #[arg(name = "Key (VKey)")] key: i32,
        ) {
            let session = get_session(state)?;
            let wnd = session.find_by_id("wnd[0]".to_owned()).map_err(|_| String::from("SAP window couldn't be requested."))?;
            match wnd {
                SAPComponent::GuiMainWindow(wnd) => wnd
                    .send_v_key(i16::try_from(key)?)
                    .map_err(|e| format!("Couldn't send VKey: {e}")),
                _ => Err(String::from("SAP window not open")),
            }?;
        }

        /// Press a button in the UI.
        #[instruction(
            id = "sap-press-button",
            lua_name = "PressButton",
            name = "Press UI Button",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn press_button(
            target: String,
        ) {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find component"))?;
            match comp {
                SAPComponent::GuiButton(b) => {
                    b.press().map_err(|e| format!("Couldn't press button: {e}"))
                }
                _ => Err(String::from("Tried to press a non-button")),
            }?;
        }

        /// Set the state of a checkbox in the UI.
        #[instruction(
            id = "sap-set-checkbox",
            lua_name = "SetCheckbox",
            name = "Checkbox: Set Value",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn set_checkbox(
            target: String,
            #[arg(id = "state", name = "Checked")] cb_state: bool,
        ) {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find component"))?;
            match comp {
                SAPComponent::GuiCheckBox(c) => c
                    .set_selected(cb_state)
                    .map_err(|e| format!("Couldn't set checkbox: {e}")),
                _ => Err(String::from("Tried to check a non-checkbox")),
            }?;
        }

        /// Set the key (selected item) of the combo box.
        #[instruction(
            id = "sap-set-combobox-key",
            lua_name = "SetComboBoxKey",
            name = "Combo Box: Set Key",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn set_combobox_key(
            target: String,
            key: String,
        ) {
            let session = get_session(state)?;
            let wnd = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            match wnd {
                SAPComponent::GuiComboBox(cmb) => cmb
                    .set_key(key)
                    .map_err(|e| format!("Can't set combo box key: {e}")),
                _ => Err("No valid target to set combo box key.".to_string()),
            }?;
        }

        /// Get the number of rows in a grid.
        #[instruction(
            id = "sap-grid-get-row-count",
            lua_name = "GetGridRowCount",
            name = "Grid: Get Row Count",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn grid_get_row_count(
            #[arg(name = "Target Grid")] target: String,
        ) -> #[output(id = "value", name = "Number of rows")] i32 {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?;
            match comp {
                SAPComponent::GuiGridView(g) => {
                    if let Ok(row_count) = g.row_count() {
                        // ! This might drop some precision in some situations!
                        Ok(row_count)
                    } else {
                        Err(String::from("The grid had no row count."))
                    }
                }
                _ => Err(String::from("The grid was invalid")),
            }?
        }

        /// Click or double click a cell.
        #[instruction(
            id = "sap-grid-click-cell",
            lua_name = "ClickGridCell",
            name = "Grid: Click or Double Click Cell",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn grid_click_cell(
            #[arg(name = "Target Grid")] target: String,
            row: i32,
            #[arg(name = "Column")] col: String,
            #[arg(name = "Double click")] double: bool,
        ) {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?;
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
        }

        /// This function adds the specified column to the collection of the selected columns.
        #[instruction(
            id = "sap-grid-select-column",
            lua_name = "SelectGridColumn",
            name = "Grid: Select Column",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_grid_column(
            #[arg(name = "Target Grid")] target: String,
            column: String,
        ) {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?;
            match comp {
                SAPComponent::GuiGridView(g) => {
                    g.select_column(column).map_err(|e| format!("The column couldn't be selected: {e}"))?;
                    Ok(())
                }
                _ => Err(String::from("The grid was invalid.")),
            }?;
        }

        /// This function removes the specified column from the collection of the selected columns.
        #[instruction(
            id = "sap-grid-deselect-column",
            lua_name = "DeselectGridColumn",
            name = "Grid: Deselect Column",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_grid_column(
            #[arg(name = "Target Grid")] target: String,
            column: String,
        ) {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?;
            match comp {
                SAPComponent::GuiGridView(g) => {
                    g.deselect_column(column).map_err(|e| format!("The column couldn't be deselected: {e}"))?;
                    Ok(())
                }
                _ => Err(String::from("The grid was invalid.")),
            }?;
        }

        /// Calling contextMenu emulates the context menu request.
        #[instruction(
            id = "sap-grid-context-menu",
            lua_name = "GridContextMenu",
            name = "Grid: Context Menu",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_grid_column(
            #[arg(name = "Target Grid")] target: String,
        ) {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?;
            match comp {
                SAPComponent::GuiGridView(g) => {
                    g.context_menu().map_err(|e| format!("Couldn't open context menu: {e}"))?;
                    Ok(())
                }
                _ => Err(String::from("The grid was invalid.")),
            }?;
        }

        /// Select an item from the control’s context menu.
        #[instruction(
            id = "sap-shell-select-context-menu-item",
            lua_name = "SelectShellContextMenuItem",
            name = "Shell: Select Context Menu Item",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn select_grid_column(
            #[arg(name = "Target")] target: String,
            function_code: String,
        ) {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?;
            match comp {
                SAPComponent::GuiGridView(g) => {
                    g.select_context_menu_item(function_code).map_err(|e| format!("Couldn't Select context menu item: {e}"))?;
                    Ok(())
                }
                // TODO Implement for all other implementors
                _ => Err(String::from("The shell was invalid.")),
            }?;
        }

        /// Get the value of a grid cell.
        #[instruction(
            id = "sap-grid-get-cell-value",
            lua_name = "GetGridCellValue",
            name = "Grid: Get Cell Value",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn grid_get_cell_value(
            #[arg(name = "Target Grid")] target: String,
            row: i32,
            #[arg(name = "Column")] col: String,
        ) -> #[output(id = "value", name = "Value")] String {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find grid view"))?;
            match comp {
                SAPComponent::GuiGridView(g) => {
                    match g.get_cell_value(row, col) {
                        Ok(value) => {
                            Ok(value)
                        }
                        Err(e) => Err(format!("The value couldn't be read: {e}")),
                    }
                }
                _ => Err(String::from("The grid view was invalid")),
            }?
        }

        /// Get the type of message displayed in the status bar shown at the bottom of the SAP window. This could be 'S' (Success), 'W' (Warning), 'E' (Error), 'A' (Abort), 'I' (Information) or '' (No Status).
        #[instruction(
            id = "sap-get-statusbar-state",
            lua_name = "GetStatusBarState",
            name = "Status Bar: Get State",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn get_statusbar_state(
            #[arg(name = "Target (usually 'wnd[0]/sbar')")] target: String,
        ) -> #[output(id = "status", name = "Status")] String {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find status bar"))?;
            match comp {
                SAPComponent::GuiStatusbar(s) => {
                    if let Ok(status) = s.message_type() {
                        Ok(status)
                    } else {
                        Err(String::from("The statusbar had no message type."))
                    }
                }
                _ => Err(String::from("The statusbar was invalid")),
            }?
        }

        /// Select a tab in a tab panel.
        #[instruction(
            id = "sap-tab-select",
            lua_name = "SelectTab",
            name = "Tab: Select",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn tab_select(
            #[arg(name = "Target Tab")] target: String,
        ) {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find tab"))?;
            match comp {
                SAPComponent::GuiTab(g) => g.select().map_err(|_| {
                    String::from("The tab couldn't be selected.")
                }),
                _ => Err(String::from("The tab was invalid")),
            }?;
        }

        /// Get the number of rows in a table.
        #[instruction(
            id = "sap-table-get-row-count",
            lua_name = "GetTableRowCount",
            name = "Table: Get Row Count",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn table_get_row_count(
            target: String,
        ) -> #[output(id = "rows", name = "Rows")] i32 {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find table"))?;
            match comp {
                SAPComponent::GuiTableControl(tab) => tab.row_count().map_err(|e| format!("Cannot read number of rows: {e}")),
                _ => Err(String::from("The table was invalid")),
            }?
        }

        /// Select a row in a table.
        #[instruction(
            id = "sap-table-row-select",
            lua_name = "SelectTableRow",
            name = "Table: Row: Select",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn table_row_select(
            target: String,
            row: i32,
        ) {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find tab"))?;
            let row_comp = match comp {
                SAPComponent::GuiTableControl(tab) => tab.get_absolute_row(row).map_err(|e| format!("Failed to get table row: {e}")),
                _ => Err(String::from("The table was invalid")),
            }?;
            row_comp.set_selected(true).map_err(|e| format!("Failed to select row: {e}"))?;
        }

        /// Get the ID of a cell that can be fed into another function's 'Target' parameter.
        #[instruction(
            id = "sap-table-cell-get-id",
            lua_name = "GetIDOfTableCell",
            name = "Table: Get ID of Cell",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn table_cell_get_id(
            target: String,
            row: i32,
            column: i32,
        ) -> #[output(id = "id", name = "Target ID")] String {
            let session = get_session(state)?;
            let comp = session.find_by_id(target).map_err(|_| String::from("Failed to find tab"))?;
            let comp = match comp {
                SAPComponent::GuiTableControl(tab) => tab.get_cell(row, column).map_err(|e| format!("Failed to get table cell: {e}")),
                _ => Err(String::from("The table was invalid")),
            }?;
            match comp {
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
            }?
        }
    }
}

unsafe impl Send for SAP {}
unsafe impl Sync for SAP {}

fn connect(state: &mut SAP) -> std::result::Result<(), String> {
    let com_instance = SAPComInstance::new().map_err(|_| "Couldn't get COM instance")?;
    let wrapper = com_instance
        .sap_wrapper()
        .map_err(|e| format!("Couldn't get SAP wrapper: {e}"))?;
    let engine = wrapper
        .scripting_engine()
        .map_err(|e| format!("Couldn't get GuiApplication instance: {e}"))?;

    let SAPComponent::GuiConnection(connection) =
        sap_scripting::GuiApplication_Impl::children(&engine)
            .map_err(|e| format!("Couldn't get GuiApplication children: {e}"))?
            .element_at(0)
            .map_err(|e| format!("Couldn't get child of GuiApplication: {e}"))?
    else {
        return Err(String::from(
            "Expected GuiConnection, but got something else!",
        ));
    };
    let SAPComponent::GuiSession(session) =
        sap_scripting::GuiConnection_Impl::children(&connection)
            .map_err(|e| format!("Couldn't get GuiConnection children: {e}"))?
            .element_at(0)
            .map_err(|e| format!("Couldn't get child of GuiConnection: {e}"))?
    else {
        return Err(String::from("Expected GuiSession, but got something else!"));
    };

    state.com_instance = Some(com_instance);
    state.session = Some(session);

    Ok(())
}

fn get_session(state: &SAP) -> std::result::Result<&GuiSession, String> {
    state
        .session
        .as_ref()
        .ok_or("GuiSession not initialised".to_string())
}
