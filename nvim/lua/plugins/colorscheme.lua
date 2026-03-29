-- nvim/lua/plugins/colorscheme.lua

return {
  {
    "Mofiqul/dracula.nvim",
    priority = 1000, -- make sure to load this before all the other start plugins
    config = function()
      vim.cmd([[colorscheme dracula]])
    end,
  },
}
