#[macro_export]
macro_rules! cast_intermediate {
    ($thing_to_cast: expr, $target_type: ty, $($via: ty),*) => {
        $thing_to_cast.downcast::<$target_type>()
            $(.or_else(|| $thing_to_cast.downcast::<$via>().map(|inner| sap_scripting::IsA::<$target_type>::upcast(&inner))))*
    };
}

#[macro_export]
macro_rules! cast_as_framewindow {
    ($comp: expr) => {
        cast_intermediate!($comp, GuiFrameWindow, GuiMainWindow, GuiModalWindow)
    };
}

#[macro_export]
macro_rules! cast_as_shell {
    ($comp: expr) => {
        cast_intermediate!(
            $comp,
            GuiShell,
            GuiBarChart,
            GuiCalendar,
            GuiChart,
            GuiColorSelector,
            GuiComboBoxControl,
            GuiContainerShell,
            GuiEAIViewer2D,
            GuiEAIViewer3D,
            GuiGraphAdapt,
            GuiGridView,
            GuiHTMLViewer,
            GuiInputFieldControl,
            GuiMap,
            GuiNetChart,
            GuiOfficeIntegration,
            GuiPicture,
            GuiSapChart,
            GuiSplit,
            GuiSplitterContainer,
            GuiStage,
            GuiTextedit,
            GuiTree
        )
    };
}

#[macro_export]
macro_rules! cast_as_vcomponent {
    ($comp: expr) => {
        cast_intermediate!(
            $comp,
            GuiVComponent,
            GuiBarChart,
            GuiBox,
            GuiButton,
            GuiCTextField,
            GuiCalendar,
            GuiChart,
            GuiCheckBox,
            GuiColorSelector,
            GuiComboBox,
            GuiComboBoxControl,
            GuiContainerShell,
            GuiCustomControl,
            GuiDialogShell,
            GuiEAIViewer2D,
            GuiEAIViewer3D,
            GuiFrameWindow,
            GuiGOSShell,
            GuiGraphAdapt,
            GuiGridView,
            GuiHTMLViewer,
            GuiInputFieldControl,
            GuiLabel,
            GuiMainWindow,
            GuiMap,
            GuiMenu,
            GuiMenubar,
            GuiModalWindow,
            GuiNetChart,
            GuiOfficeIntegration,
            GuiOkCodeField,
            GuiPasswordField,
            GuiPicture,
            GuiRadioButton,
            GuiSapChart,
            GuiScrollContainer,
            GuiShell,
            GuiSimpleContainer,
            GuiSplit,
            GuiSplitterContainer,
            GuiStage,
            GuiStatusPane,
            GuiStatusbar,
            GuiTab,
            GuiTabStrip,
            GuiTableControl,
            GuiTextField,
            GuiTextedit,
            GuiTitlebar,
            GuiToolbar,
            GuiTree,
            GuiUserArea,
            GuiVContainer,
            GuiVHViewSwitch
        )
    };
}
