#![warn(clippy::pedantic)]

use std::fs;

#[allow(clippy::wildcard_imports)]
use sap_scripting::*;
use testangel_engine::{Evidence, EvidenceContent, engine};

mod macros;

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
            let comp = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            let wnd = cast_as_framewindow!(comp).ok_or("No valid target to screenshot.")?;
            let path = wnd.hard_copy("evidence.png".to_string(), 2)
                .map_err(|e| format!("Can't screenshot: {e}"))?;
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
            comp.r_type()?
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
            let comp = cast_as_vcomponent!(comp).ok_or("No valid target to visualise.")?;
            comp.visualize(true).map_err(|e| format!("Failed to visualize: {e}"))?;
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
            let comp = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            let vcomp = cast_as_vcomponent!(comp).ok_or("No valid target to set text.")?;
            vcomp.set_text(value).map_err(|e| format!("Can't set text: {e}"))?;
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
            let comp = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?;
            let vcomp = cast_as_vcomponent!(comp).ok_or("No valid target to get text.")?;
            vcomp.text().map_err(|e| format!("Can't get text: {e}"))?
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
            let comp = session.find_by_id("wnd[0]".to_owned()).map_err(|_| String::from("SAP window couldn't be requested."))?;
            let fw = cast_as_framewindow!(comp).ok_or("Window invalid")?;
            fw.send_v_key(i16::try_from(key)?).map_err(|e| format!("Couldn't send VKey: {e}"))?;
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
            let b: GuiButton = session.find_by_id(target).map_err(|_| String::from("Failed to find component"))?
                .downcast().ok_or("Tried to press a non-button")?;
            b.press().map_err(|e| format!("Couldn't press button: {e}"))?;
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
            let c: GuiCheckBox = session.find_by_id(target).map_err(|_| String::from("Failed to find component"))?
                .downcast().ok_or("Tried to check a non-checkbox")?;
            c.set_selected(cb_state).map_err(|e| format!("Couldn't set checkbox: {e}"))?;
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
            let cmb: GuiComboBox = session.find_by_id(target.clone()).map_err(|_| format!("Couldn't find {target}."))?
                .downcast().ok_or("No valid target to set combo box key")?;
            cmb.set_key(key).map_err(|e| format!("Can't set combo box key: {e}"))?;
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
            let grid: GuiGridView = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?
                .downcast().ok_or("The grid was invalid")?;
            grid.row_count()?
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
            let grid: GuiGridView = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?
                .downcast().ok_or("The grid was invalid")?;
            grid.set_current_cell(row, col).map_err(|e| format!("Couldn't select cell in grid: {e}"))?;
            if double {
                grid.double_click_current_cell().map_err(|e| {
                    format!("The grid couldn't be double clicked: {e}")
                })?;
            } else {
                grid.click_current_cell().map_err(|e| {
                    format!("The grid couldn't be clicked: {e}")
                })?;
            }
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
            let grid: GuiGridView = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?
                .downcast().ok_or("The grid was invalid")?;
            grid.select_column(column).map_err(|e| format!("The column couldn't be selected: {e}"))?;
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
            let grid: GuiGridView = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?
                .downcast().ok_or("The grid was invalid")?;
            grid.deselect_column(column).map_err(|e| format!("The column couldn't be deselected: {e}"))?;
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
            let grid: GuiGridView = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?
                .downcast().ok_or("The grid was invalid")?;
            grid.context_menu().map_err(|e| format!("Couldn't open context menu: {e}"))?;
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
            let grid: GuiGridView = session.find_by_id(target).map_err(|_| String::from("Failed to find grid"))?
                .downcast().ok_or("The grid was invalid")?;
            grid.get_cell_value(row, col)?
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
            let shell = cast_as_shell!(comp).ok_or("The shell was invalid")?;
            shell.select_context_menu_item(function_code).map_err(|e| format!("Couldn't select context menu item: {e}"))?;
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
            let s: GuiStatusbar = session.find_by_id(target).map_err(|_| String::from("Failed to find status bar"))?
                .downcast().ok_or("The statusbar was invalid")?;
            s.message_type()?
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
            let tab: GuiTab = session.find_by_id(target).map_err(|_| String::from("Failed to find tab"))?
                .downcast().ok_or("The tab was invalid")?;
            tab.select()?;
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
            let tab: GuiTableControl = session.find_by_id(target).map_err(|_| String::from("Failed to find table"))?
                .downcast().ok_or("The table was invalid")?;
            tab.row_count()?
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
            let tab: GuiTableControl = session.find_by_id(target).map_err(|_| String::from("Failed to find table"))?
                .downcast().ok_or("The table was invalid")?;
            let row = tab.get_absolute_row(row).map_err(|e| format!("Failed to get table row: {e}"))?;
            row.set_selected(true).map_err(|e| format!("Failed to select row: {e}"))?;
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
            let tab: GuiTableControl = session.find_by_id(target).map_err(|_| String::from("Failed to find table"))?
                .downcast().ok_or("The table was invalid")?;
            let comp = tab.get_cell(row, column).map_err(|e| format!("Failed to get table cell: {e}"))?;
            comp.id()?
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

    let connection: GuiConnection = sap_scripting::GuiApplicationExt::children(&engine)
        .map_err(|e| format!("Couldn't get GuiApplication children: {e}"))?
        .element_at(0)
        .map_err(|e| format!("Couldn't get child of GuiApplication: {e}"))?
        .downcast()
        .ok_or("Expected GuiConnection, but got something else!")?;
    let session: GuiSession = sap_scripting::GuiConnectionExt::children(&connection)
        .map_err(|e| format!("Couldn't get GuiConnection children: {e}"))?
        .element_at(0)
        .map_err(|e| format!("Couldn't get child of GuiConnection: {e}"))?
        .downcast()
        .ok_or("Expected GuiSession, but got something else!")?;

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
