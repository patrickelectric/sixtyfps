const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  module: {
    rules: [
      { test: /\.60$/, use: "../../api/sixtyfps-web/webpack-loader" }
    ]
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new CopyPlugin([
      path.resolve(__dirname, "static")
    ])
  ]
};
