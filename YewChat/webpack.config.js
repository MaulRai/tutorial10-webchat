const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const distPath = path.resolve(__dirname, 'dist');

module.exports = {
  mode: 'production',
  entry: './bootstrap.js',
  output: {
    path: distPath,
    filename: 'yewchat.js',
    webassemblyModuleFilename: 'yewchat_bg.wasm',
  },
  devServer: {
    port: 8000,
  },
  experiments: {
    asyncWebAssembly: true,
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [{ from: './static', to: distPath }],
    }),
    new WasmPackPlugin({
      crateDirectory: '.',
      extraArgs: '-- --features wee_alloc',
      outName: 'yewchat',
    }),
  ],
};
