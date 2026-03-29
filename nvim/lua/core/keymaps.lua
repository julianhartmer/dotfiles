-- nvim/lua/core/keymaps.lua

local keymap = vim.keymap

-- Clear search highlights
keymap.set("n", "<leader>nh", ":nohl<CR>", { desc = "Clear search highlights" })

-- Window management
keymap.set("n", "<leader>v", "<C-w>v", { desc = "Split window vertically" })
keymap.set("n", "<leader>x", "<C-w>s", { desc = "Split window horizontally" })
keymap.set("n", "<leader>se", "<C-w>=", { desc = "Make splits equal size" })
keymap.set("n", "<leader>q", "<cmd>close<CR>", { desc = "Close current split" })

-- General
keymap.set("n", "<leader>w", "<cmd>w<CR>", { desc = "Save file" })
keymap.set("n", "<leader>c", "<cmd>bd<CR>", { desc = "Close current buffer" })
keymap.set("n", "<leader>l", "<cmd>bnext<CR>", { desc = "Next buffer" })
keymap.set("n", "<leader>h", "<cmd>bprev<CR>", { desc = "Previous buffer" })
