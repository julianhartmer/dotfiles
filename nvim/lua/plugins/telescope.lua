-- nvim/lua/plugins/telescope.lua

return {
  {
    "nvim-telescope/telescope.nvim",
    branch = "master",
    dependencies = {
      "nvim-lua/plenary.nvim",
      "nvim-tree/nvim-web-devicons",
    },
    config = function()
      local telescope = require("telescope")
      local builtin = require("telescope.builtin")
      local actions = require("telescope.actions")

      telescope.setup({
        defaults = {
          path_display = { "truncate" },
          mappings = {
            i = {
              ["<C-v>"] = actions.select_vertical,
              ["<C-x>"] = actions.select_horizontal,
            },
            n = {
              ["<C-v>"] = actions.select_vertical,
              ["<C-x>"] = actions.select_horizontal,
            },
          },
        },
      })

      -- Keymaps
      vim.keymap.set("n", "<leader>ff", builtin.find_files, { desc = "Fuzzy find files in cwd" })
      vim.keymap.set("n", "<leader>fr", builtin.oldfiles, { desc = "Fuzzy find recent files" })
      vim.keymap.set("n", "<leader>fs", builtin.live_grep, { desc = "Find string in cwd" })
      vim.keymap.set("n", "<leader>fc", builtin.grep_string, { desc = "Find string under cursor" })
      vim.keymap.set("n", "<leader>fb", builtin.buffers, { desc = "Fuzzy find open buffers" })
    end,
  },
}
