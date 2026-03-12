#[derive(Clone, Copy, PartialEq)]
pub enum Lang {
    Zh,
    En,
}

pub struct Texts {
    // Common
    pub app_title: &'static str,

    // Platform select
    pub select_platform: &'static str,
    pub desc_chatgpt: &'static str,
    pub desc_claude_web: &'static str,
    pub desc_claude_code: &'static str,
    pub desc_cursor: &'static str,

    // Token
    pub input_label: &'static str,
    pub token_label: &'static str,
    pub hint_label: &'static str,
    pub hint_chatgpt: &'static str,
    pub hint_claude: &'static str,

    // Filter
    pub filter_title: &'static str,
    pub keyword_mode: &'static str,
    pub include: &'static str,
    pub exclude: &'static str,
    pub keywords: &'static str,
    pub keywords_hint: &'static str,
    pub start_date: &'static str,
    pub end_date: &'static str,
    pub filter_empty_hint: &'static str,
    pub space_to_toggle: &'static str,

    // Loading
    pub loading_msg: &'static str,

    // Conversation list
    pub conversations: &'static str,
    pub total_label: &'static str,
    pub selected_label: &'static str,

    // Download
    pub downloading_title: &'static str,
    pub downloading_msg: &'static str,
    pub preparing_msg: &'static str,

    // Done
    pub done_title: &'static str,
    pub done_msg: &'static str,
    pub exported_to: &'static str,

    // Error
    pub error_title: &'static str,

    // Help fragments
    pub help_nav: &'static str,
    pub help_enter_select: &'static str,
    pub help_enter_confirm: &'static str,
    pub help_enter_download: &'static str,
    pub help_enter_fetch: &'static str,
    pub help_enter_apply: &'static str,
    pub help_enter_back: &'static str,
    pub help_esc_back: &'static str,
    pub help_esc_cancel: &'static str,
    pub help_q_quit: &'static str,
    pub help_tab_switch: &'static str,
    pub help_tab_show_hide: &'static str,
    pub help_space_toggle: &'static str,
    pub help_space_mode: &'static str,
    pub help_a_select_all: &'static str,
    pub help_f_filter: &'static str,
}

pub fn texts(lang: Lang) -> &'static Texts {
    match lang {
        Lang::Zh => &ZH,
        Lang::En => &EN,
    }
}

static ZH: Texts = Texts {
    app_title: "LLM 对话导出工具",

    select_platform: " 选择平台 ",
    desc_chatgpt: "OpenAI ChatGPT (Access Token)",
    desc_claude_web: "Anthropic Claude.ai (Session Key)",
    desc_claude_code: "本地 Claude Code 会话记录",
    desc_cursor: "本地 Cursor 编辑器会话记录",

    input_label: "输入",
    token_label: " Token ",
    hint_label: " 提示 ",
    hint_chatgpt: "打开 chatgpt.com -> F12 -> Network 标签\n找到任意请求 -> 复制 Authorization 的值 (去掉 'Bearer ' 前缀)",
    hint_claude: "打开 claude.ai -> F12 -> Application -> Cookies\n复制 sessionKey 的值",

    filter_title: " 筛选设置 ",
    keyword_mode: "关键词模式:",
    include: "包含",
    exclude: "不包含",
    keywords: "关键词:    ",
    keywords_hint: "(逗号分隔, 匹配标题)",
    start_date: "开始日期:  ",
    end_date: "结束日期:  ",
    filter_empty_hint: "留空则不筛选该项",
    space_to_toggle: "(空格切换)",

    loading_msg: "正在加载会话列表...",

    conversations: " 会话列表 ",
    total_label: "共",
    selected_label: "已选",

    downloading_title: " 下载中 ",
    downloading_msg: "下载中...",
    preparing_msg: "准备中...",

    done_title: " 完成 ",
    done_msg: "完成！",
    exported_to: "已导出到:",

    error_title: " 错误 ",

    help_nav: "上下",
    help_enter_select: "Enter:选择",
    help_enter_confirm: "Enter:确认",
    help_enter_download: "Enter:下载",
    help_enter_fetch: "Enter:获取并筛选",
    help_enter_apply: "Enter:应用筛选",
    help_enter_back: "Enter:返回",
    help_esc_back: "Esc:返回",
    help_esc_cancel: "Esc:取消",
    help_q_quit: "q:退出",
    help_tab_switch: "Tab:切换",
    help_tab_show_hide: "Tab:显示/隐藏",
    help_space_toggle: "空格:选择",
    help_space_mode: "空格:切换模式",
    help_a_select_all: "a:全选",
    help_f_filter: "f:筛选",
};

static EN: Texts = Texts {
    app_title: "LLM Chat Export",

    select_platform: " Select Platform ",
    desc_chatgpt: "OpenAI ChatGPT (Access Token)",
    desc_claude_web: "Anthropic Claude.ai (Session Key)",
    desc_claude_code: "Local Claude Code sessions",
    desc_cursor: "Local Cursor editor sessions",

    input_label: "Input",
    token_label: " Token ",
    hint_label: " Hint ",
    hint_chatgpt: "Open chatgpt.com -> F12 -> Network tab\nFind any request -> Copy Authorization header value (remove 'Bearer ' prefix)",
    hint_claude: "Open claude.ai -> F12 -> Application -> Cookies\nCopy the 'sessionKey' cookie value",

    filter_title: " Filter Settings ",
    keyword_mode: "Keyword Mode:",
    include: "Include",
    exclude: "Exclude",
    keywords: "Keywords:    ",
    keywords_hint: "(comma separated, matches title)",
    start_date: "Start Date:  ",
    end_date: "End Date:    ",
    filter_empty_hint: "Leave empty to skip filter",
    space_to_toggle: "(Space to toggle)",

    loading_msg: "Loading conversations...",

    conversations: " Conversations ",
    total_label: "Total",
    selected_label: "Selected",

    downloading_title: " Downloading ",
    downloading_msg: "Downloading...",
    preparing_msg: "Preparing...",

    done_title: " Complete ",
    done_msg: "Done!",
    exported_to: "Exported to:",

    error_title: " Error ",

    help_nav: "Up/Down",
    help_enter_select: "Enter: Select",
    help_enter_confirm: "Enter: Confirm",
    help_enter_download: "Enter: Download",
    help_enter_fetch: "Enter: Fetch & Filter",
    help_enter_apply: "Enter: Apply Filter",
    help_enter_back: "Enter: Back",
    help_esc_back: "Esc: Back",
    help_esc_cancel: "Esc: Cancel",
    help_q_quit: "q: Quit",
    help_tab_switch: "Tab: Switch",
    help_tab_show_hide: "Tab: Show/Hide",
    help_space_toggle: "Space: Toggle",
    help_space_mode: "Space: Toggle Mode",
    help_a_select_all: "a: Select All",
    help_f_filter: "f: Filter",
};
