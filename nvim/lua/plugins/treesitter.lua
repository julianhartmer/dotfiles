-- nvim/lua/plugins/treesitter.lua

return {
  {
    "nvim-treesitter/nvim-treesitter",
    event = { "BufReadPre", "BufNewFile" },
    build = ":TSUpdate",
    config = function()
      local config = require("nvim-treesitter.config")

      -- Set parsers to install
      config.ensure_installed = { "lua", "vim", "vimdoc", "javascript", "typescript", "python", "bash", "markdown", "markdown_inline" }
      config.sync_install = false
      config.auto_install = true

      -- Enable modules
      vim.treesitter.language.register('markdown', 'mdx')

      -- Configure highlighting
      vim.opt.foldmethod = "expr"
      vim.opt.foldexpr = "nvim_treesitter#foldexpr()"
      vim.opt.foldenable = false
    end,
  },
}
