const path = require('path');
const crypto = require('crypto');
const { execSync } = require('child_process');
const webpack = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

// Identifies this build, so a running page can tell whether it is the one
// currently deployed. GitHub Actions exports GITHUB_SHA; locally fall back
// to the git HEAD, and to a random value if neither is available (a
// detached tarball, say) so two builds never collide.
function buildId() {
  if (process.env.GITHUB_SHA) return process.env.GITHUB_SHA.slice(0, 12);
  try {
    return execSync('git rev-parse --short=12 HEAD', { stdio: ['ignore', 'pipe', 'ignore'] })
      .toString()
      .trim();
  } catch {
    return crypto.randomBytes(6).toString('hex');
  }
}

const BUILD_ID = buildId();

/**
 * Writes the build id to a small `version.json` alongside the bundle.
 *
 * GitHub Pages serves HTML with `cache-control: max-age=600` and provides
 * no way to change that — there is no `_headers` file or equivalent. So for
 * up to ten minutes after a deploy a returning visitor can be running the
 * previous HTML, and therefore the previous bundle, with no indication
 * anything is stale. The page fetches this file to notice that itself; see
 * src/version-check.js.
 */
class EmitVersionPlugin {
  apply(compiler) {
    const { Compilation, sources } = compiler.webpack;
    compiler.hooks.thisCompilation.tap('EmitVersionPlugin', (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: 'EmitVersionPlugin',
          stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
        },
        () => {
          compilation.emitAsset(
            'version.json',
            new sources.RawSource(JSON.stringify({ buildId: BUILD_ID })),
          );
        },
      );
    });
  }
}

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
    new webpack.DefinePlugin({
      // The page compares this against the deployed version.json.
      __BUILD_ID__: JSON.stringify(BUILD_ID),
    }),
    new EmitVersionPlugin(),
    new HtmlWebpackPlugin({
      template: './index.html',
      favicon: false,
    }),
    new CopyWebpackPlugin({
      patterns: [
        // NOTE: the .wasm binary is deliberately NOT copied here. The glue in
        // pkg/mortgage_wasm.js resolves it with
        // `new URL('mortgage_wasm_bg.wasm', import.meta.url)`, a pattern
        // webpack recognises and rewrites — it emits its own content-hashed
        // copy and points the bundle at that. Copying it as well shipped the
        // binary twice (~858 KB of dead weight in dist/), since nothing ever
        // requested the unhashed name.
        //
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
