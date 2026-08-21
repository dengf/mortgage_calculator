const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = {
  entry: './src/index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle.[contenthash].js',
    clean: true,
  },
  // wasm-pack's `--target web` output does its own `fetch` + instantiate of
  // the .wasm binary (see pkg/mortgage_wasm.js), so webpack's native wasm
  // module linking must stay off — turning it on made webpack intercept
  // that internal import and hand back a differently-shaped module,
  // corrupting values returned across the JS/wasm boundary for some
  // (not all) exported functions.
  module: {
    rules: [
      {
        test: /\.(js|jsx)$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: ['@babel/preset-env', '@babel/preset-react'],
          },
        },
      },
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader'],
      },
    ],
  },
  resolve: {
    extensions: ['.js', '.jsx', '.wasm'],
    alias: {
      pkg: path.resolve(__dirname, 'pkg'),
    },
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: './index.html',
      favicon: false,
    }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: 'pkg/*.wasm',
          to: '[name][ext]',
          noErrorOnMissing: true,
        },
        // Favicons and the web app manifest, referenced by name from
        // index.html — so they must land in dist/ with their names intact
        // rather than being hashed like bundled assets.
        {
          from: 'static',
          to: '.',
        },
      ],
    }),
  ],
  devServer: {
    static: {
      directory: path.join(__dirname, 'dist'),
    },
    compress: true,
    port: 3001,
    hot: true,
  },
  performance: {
    hints: false,
    maxEntrypointSize: 512000,
    maxAssetSize: 512000,
  },
};
