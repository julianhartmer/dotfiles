return {
  "epwalsh/obsidian.nvim",
  version = "*",  -- recommended, use latest release instead of latest commit
  lazy = false,
  ft = "markdown",
  dependencies = {
    "nvim-lua/plenary.nvim",
  },
  keys = {
    { "<leader>oo", "<cmd>ObsidianToday<CR>",        desc = "Open today's daily note" },
    { "<leader>oy", "<cmd>ObsidianYesterday<CR>",    desc = "Open yesterday's daily note" },
    { "<leader>os", "<cmd>ObsidianSearch<CR>",       desc = "Search notes" },
    { "<leader>oq", "<cmd>ObsidianQuickSwitch<CR>",  desc = "Quick switch note" },
    { "<leader>on", "<cmd>ObsidianNew<CR>",          desc = "New note" },
    { "<leader>ot", "<cmd>ObsidianTemplate<CR>",     desc = "Insert template" },
    { "<leader>ob", "<cmd>ObsidianBacklinks<CR>",    desc = "Show backlinks" },
    { "<leader>ol", "<cmd>ObsidianLinks<CR>",        desc = "Show links in note" },
    { "<leader>of", "<cmd>ObsidianFollowLink<CR>",   desc = "Follow link under cursor" },
  },
  opts = {
    workspaces = {
      {
        name = "work",
        path = os.getenv("OBSIDIAN_VAULT") or "~/notes/"
      },
    },

    templates = {
        folder = os.getenv("OBSIDIAN_TEMPLATES") or "_Templates",
        date_format = "%Y-%m-%d-%a",
        time_format = "%H:%M",
    },

    daily_notes = {
      folder = "Journal",
      template = "Daily Note Template",
    },
  },
}
