// @ts-check
/** @type {import('@typescript-eslint/utils').TSESLint.Linter.Config} */
module.exports = {
  root: true,
  parser: "@typescript-eslint/parser",
  parserOptions: {
    project: true,
    tsconfigRootDir: __dirname,
  },
  plugins: ["@typescript-eslint", "solid"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/strict",
    "plugin:@typescript-eslint/strict-type-checked",
    "plugin:solid/typescript",
    "prettier",
  ],
  env: {
    browser: true,
    es2022: true,
  },
  ignorePatterns: ["dist/", "src-tauri/", "*.cjs", "src/bindings.ts"],
};
