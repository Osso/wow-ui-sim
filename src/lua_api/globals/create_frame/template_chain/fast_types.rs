#[derive(Clone)]
pub(crate) enum FastHandlerRef<'a> {
    NoOp,
    InlineMethod {
        handler_name: &'static str,
        body: &'a str,
    },
    Sequence2(Box<(FastHandlerRef<'a>, FastHandlerRef<'a>)>),
    Sequence3(Box<(FastHandlerRef<'a>, FastHandlerRef<'a>, FastHandlerRef<'a>)>),
    Sequence4(
        Box<(
            FastHandlerRef<'a>,
            FastHandlerRef<'a>,
            FastHandlerRef<'a>,
            FastHandlerRef<'a>,
        )>,
    ),
    ConditionalGlobalFieldEqualsStringThen {
        target_path: &'a str,
        field: &'a str,
        value: &'a str,
        then_ref: Box<FastHandlerRef<'a>>,
    },
    ConditionalSelfNoArgsMethod {
        method_name: &'a str,
        then_ref: Box<FastHandlerRef<'a>>,
        else_ref: Box<FastHandlerRef<'a>>,
    },
    ConditionalNotSelfNoArgsMethodThen {
        method_name: &'a str,
        then_ref: Box<FastHandlerRef<'a>>,
    },
    ConditionalSelfFieldTruthy {
        field: &'a str,
        then_ref: Box<FastHandlerRef<'a>>,
        else_ref: Box<FastHandlerRef<'a>>,
    },
    Method(&'a str),
    MethodWithBoolArg {
        method_name: &'a str,
        value: bool,
    },
    MethodWithNumberArg {
        method_name: &'a str,
        value: f64,
    },
    MethodWithTwoNumberArgs {
        method_name: &'a str,
        first: f64,
        second: f64,
    },
    MethodWithStringArg {
        method_name: &'a str,
        arg: &'a str,
    },
    SelfFieldMethod {
        field: &'a str,
        method_name: &'a str,
    },
    SelfFieldMethodWithStringArg {
        field: &'a str,
        method_name: &'a str,
        arg: &'a str,
    },
    SelfFieldMethodWithNumberArg {
        field: &'a str,
        method_name: &'a str,
        value: f64,
    },
    SelfFieldMethodWithGlobalArg {
        field: &'a str,
        method_name: &'a str,
        arg_path: &'a str,
    },
    SelfFieldMethodWithSelfFieldArg {
        field: &'a str,
        method_name: &'a str,
        arg_field: &'a str,
    },
    SelfFieldMethodWithStringNumberNumberArgs {
        field: &'a str,
        method_name: &'a str,
        first: &'a str,
        second: f64,
        third: f64,
    },
    SelfFieldMethodWithStringSelfStringNumberNumberArgs {
        field: &'a str,
        method_name: &'a str,
        first: &'a str,
        third: &'a str,
        fourth: f64,
        fifth: f64,
    },
    ParentMethod(&'a str),
    ParentMethodWithStringArg {
        method_name: &'a str,
        arg: &'a str,
    },
    ParentFieldMethodWithSelfNoArgMethodResult {
        field: &'a str,
        method_name: &'a str,
        self_method_name: &'a str,
    },
    GrandparentFieldMethod {
        field: &'a str,
        method_name: &'a str,
    },
    GrandparentMethod(&'a str),
    GlobalMethod {
        target_path: &'a str,
        method_name: &'a str,
    },
    GlobalMethodWithSelfArg {
        target_path: &'a str,
        method_name: &'a str,
    },
    GlobalMethodWithSelfStringArg {
        target_path: &'a str,
        method_name: &'a str,
        arg: &'a str,
    },
    GlobalMethodWithSelfStringNumberNumberArgs {
        target_path: &'a str,
        method_name: &'a str,
        first: &'a str,
        second: f64,
        third: f64,
    },
    GlobalMethodWithStringArg {
        target_path: &'a str,
        method_name: &'a str,
        arg: &'a str,
    },
    GlobalMethodWithGlobalArg {
        target_path: &'a str,
        method_name: &'a str,
        arg_path: &'a str,
    },
    GlobalMethodWithStringGlobalBoolArgs {
        target_path: &'a str,
        method_name: &'a str,
        first: &'a str,
        second_arg_path: &'a str,
        third: bool,
    },
    GlobalMethodWithGlobalThreeGlobalBoolArgs {
        target_path: &'a str,
        method_name: &'a str,
        first_arg_path: &'a str,
        second_arg_path: &'a str,
        third_arg_path: &'a str,
        fourth_arg_path: &'a str,
        fifth: bool,
    },
    GlobalMethodWithGlobalNilNilNilNilBoolArgs {
        target_path: &'a str,
        method_name: &'a str,
        first_arg_path: &'a str,
        sixth: bool,
    },
    GlobalMethodWithFourGlobalArgs {
        target_path: &'a str,
        method_name: &'a str,
        first_arg_path: &'a str,
        second_arg_path: &'a str,
        third_arg_path: &'a str,
        fourth_arg_path: &'a str,
    },
    GlobalMethodWithStringStringFunctionResultAndThreeNumberArgs {
        target_path: &'a str,
        method_name: &'a str,
        function_name: &'a str,
        first: &'a str,
        second: &'a str,
        third: f64,
        fourth: f64,
        fifth: f64,
    },
    GlobalMethodWithGlobalStringFunctionResultAndThreeNumberArgs {
        target_path: &'a str,
        method_name: &'a str,
        function_name: &'a str,
        first_arg_path: &'a str,
        second: &'a str,
        third: f64,
        fourth: f64,
        fifth: f64,
    },
    GlobalMethodWithGlobalSelfMethodSelfMethodBoolArgs {
        target_path: &'a str,
        method_name: &'a str,
        first_arg_path: &'a str,
        second_self_method: &'a str,
        third_self_method: &'a str,
        fourth: bool,
    },
    GlobalMethodWithSelfIdArg {
        target_path: &'a str,
        method_name: &'a str,
    },
    GlobalMethodWithSelfFieldArg {
        target_path: &'a str,
        method_name: &'a str,
        field: &'a str,
    },
    GlobalTooltipSetOwnerThenSetText {
        target_path: &'a str,
        anchor: &'a str,
        text_path: &'a str,
        red_path: &'a str,
        green_path: &'a str,
        blue_path: &'a str,
        wrap: bool,
    },
    GlobalTooltipSetOwnerThenSetTextLiteral {
        target_path: &'a str,
        anchor: &'a str,
        text: &'a str,
        red: f64,
        green: f64,
        blue: f64,
    },
    ConditionalTooltip {
        target_path: &'a str,
        field: &'a str,
        anchor: &'a str,
        red_path: &'a str,
        green_path: &'a str,
        blue_path: &'a str,
    },
    ToggleGlobalVisibility {
        target_path: &'a str,
    },
    NamedGlobalMethodWithGlobalArg {
        suffix: &'a str,
        method_name: &'a str,
        arg_path: &'a str,
    },
    GlobalMethodThenAssignLiteral {
        target_path: &'a str,
        method_name: &'a str,
        field: &'a str,
        value: FastLiteralValue<'a>,
    },
    Function(&'a str),
    FunctionWithSelfArg(&'a str),
    FunctionNoArgs(&'a str),
    FunctionWithSelfIdArg(&'a str),
    FunctionWithStringArg {
        function_name: &'a str,
        arg: &'a str,
    },
    FunctionWithStringNumberArgs {
        function_name: &'a str,
        first: &'a str,
        second: f64,
    },
    FunctionWithTwoGlobalNumberArgs {
        function_name: &'a str,
        first_arg_path: &'a str,
        second_arg_path: &'a str,
        third: f64,
    },
    FunctionWithStringNilNilGlobalArgs {
        function_name: &'a str,
        first: &'a str,
        fourth: &'a str,
    },
    FunctionWithNoArgFunctionResult {
        function_name: &'a str,
        arg_function_name: &'a str,
    },
    FunctionWithGlobalMethodNoArgsResult {
        function_name: &'a str,
        target_path: &'a str,
        method_name: &'a str,
    },
    FunctionWithSelfNoArgsMethodResult {
        function_name: &'a str,
        method_name: &'a str,
    },
    FunctionWithSelfStringArg {
        function_name: &'a str,
        arg: &'a str,
    },
    FunctionWithStringSelfStringNumberNumberArgs {
        function_name: &'a str,
        first: &'a str,
        third: &'a str,
        fourth: f64,
        fifth: f64,
    },
    FunctionWithSelfNumberArg {
        function_name: &'a str,
        value: f64,
    },
    FunctionWithNumberArg {
        function_name: &'a str,
        value: f64,
    },
    FunctionWithGlobalArg {
        function_name: &'a str,
        arg_path: &'a str,
    },
    FunctionWithTwoGlobalArgs {
        function_name: &'a str,
        first_arg_path: &'a str,
        second_arg_path: &'a str,
    },
    FunctionWithThreeGlobalArgs {
        function_name: &'a str,
        first_arg_path: &'a str,
        second_arg_path: &'a str,
        third_arg_path: &'a str,
    },
    FunctionWithGlobalSelfMethodSelfMethodBoolArgs {
        function_name: &'a str,
        first_arg_path: &'a str,
        second_self_method: &'a str,
        third_self_method: &'a str,
        fourth: bool,
    },
    FunctionWithStringGlobalBoolArg {
        function_name: &'a str,
        first: &'a str,
        second_arg_path: &'a str,
        third: bool,
    },
    FunctionWithGlobalAndSelfArg {
        function_name: &'a str,
        global_arg_path: &'a str,
    },
    FunctionWithGlobalAndSelfIdArg {
        function_name: &'a str,
        global_arg_path: &'a str,
    },
    FunctionWithParentFieldArg {
        function_name: &'a str,
        field: &'a str,
    },
    FunctionWithParentFieldAndNestedParentFieldMethodResult {
        function_name: &'a str,
        first_field: &'a str,
        second_field: &'a str,
        third_field: &'a str,
        method_name: &'a str,
    },
    FunctionWithSelfAndParentFieldArg {
        function_name: &'a str,
        field: &'a str,
    },
    CheckedAssignmentThenCallbacks {
        target_path: &'a str,
        field: &'a str,
        on_change_function: &'a str,
        on_sound_function: &'a str,
    },
    CheckedAssignments3ThenCallbacks {
        first_target_path: &'a str,
        first_field: &'a str,
        second_target_path: &'a str,
        second_field: &'a str,
        third_target_path: &'a str,
        third_field: &'a str,
        on_change_function: &'a str,
        on_sound_function: &'a str,
    },
    CheckedAssignmentThenTwoCallbacks {
        target_path: &'a str,
        field: &'a str,
        first_callback: &'a str,
        second_callback: &'a str,
        on_sound_function: &'a str,
    },
    CheckedNumberAssignmentThenCallbacks {
        target_path: &'a str,
        field: &'a str,
        value: f64,
        on_change_function: &'a str,
        on_sound_function: &'a str,
    },
    ParentFieldLocalToggleShown {
        field: &'a str,
    },
    ParentFieldLocalClickIfEnabled {
        field: &'a str,
    },
    MethodThenUncheckedParentFieldClearAndShowText {
        method_name: &'a str,
        field: &'a str,
    },
    ConditionalSelfGetTextNonEmptyThenParentMethodWithSelfGetTextAndClear {
        method_name: &'a str,
    },
    ConditionalSelfTextEmptyShowTextChild,
    LocalGlobalPathConditionalMethod {
        target_path: &'a str,
        method_name: &'a str,
    },
    GrandparentMethodWithNotSelfCheckedArg {
        method_name: &'a str,
    },
    FunctionWithSelfGetTextResult(&'a str),
    FunctionWithParentArg(&'a str),
    FunctionWithGrandparentArg(&'a str),
    FunctionWithParentIdArg(&'a str),
    FunctionWithEventVarargs(&'a str),
    FunctionWithButton(&'a str),
    FunctionWithElapsed(&'a str),
    RegisterForClicks {
        first: &'a str,
        second: Option<&'a str>,
        third: Option<&'a str>,
    },
    RegisterForDrag(&'a str),
    SetAlpha(f64),
    SetFrameLevelFromParent(i32),
    AssignAncestorRef {
        field: &'a str,
        depth: usize,
    },
    AssignLiteral {
        field: &'a str,
        value: FastLiteralValue<'a>,
    },
    AssignGlobalFieldLiteral {
        target_path: &'a str,
        field: &'a str,
        value: FastLiteralValue<'a>,
    },
    AssignNestedLiteral {
        parent_field: &'a str,
        field: &'a str,
        value: FastLiteralValue<'a>,
    },
    AssignNestedGlobalPairTable {
        parent_field: &'a str,
        field: &'a str,
        first_path: &'a str,
        second_path: &'a str,
    },
    AssignParentField {
        field: &'a str,
        value: FastLiteralValue<'a>,
    },
}

#[derive(Copy, Clone)]
pub(crate) enum FastLiteralValue<'a> {
    Global(&'a str),
    Number(f64),
    Nil,
    Bool(bool),
}

#[derive(Clone)]
pub(crate) enum FastScriptInstall<'a> {
    Set(FastHandlerRef<'a>),
    Intrinsic {
        handler: FastHandlerRef<'a>,
        new_first: bool,
    },
    Chain {
        handler: FastHandlerRef<'a>,
        new_first: bool,
    },
}
