import {defineConfig} from "vite";
import vue from "@vitejs/plugin-vue";
import * as path from "path";
import VueSetupExtend from 'vite-plugin-vue-setup-extend'
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill'
import { NodeModulesPolyfillPlugin } from '@esbuild-plugins/node-modules-polyfill'
import rollupNodePolyFill from 'rollup-plugin-polyfill-node'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [vue(), VueSetupExtend],
    // 设置路径别名
    resolve: {
        alias: [
            {
                find: '@',
                replacement: path.resolve('./src')
            }
        ]
    },
    optimizeDeps: {
        include: ['ethers', 'vue', 'vue-router'],
        esbuildOptions: {
            // Node.js global to browser globalThis
            define: {
                global: 'globalThis'
            },
            // Enable esbuild polyfill plugins
            plugins: [
                NodeGlobalsPolyfillPlugin({
                    buffer: true,
                    process: true,
                }),
                NodeModulesPolyfillPlugin()
            ]
        }
    },
    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    // prevent vite from obscuring rust errors
    clearScreen: false,
    // tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
    },
    // to make use of `TAURI_DEBUG` and other env variables
    // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
    envPrefix: ["VITE_", "TAURI_"],
    build: {
        rollupOptions: {
            plugins: [
                rollupNodePolyFill()
            ],
            output: {
                manualChunks: {
                // 将大型依赖分离到独立chunk
                'vendor-crypto': ['ethers'],
                'vendor-ui': ['vue', 'vue-router', '@arco-design/web-vue'],
                'vendor-utils': ['xlsx', 'pinia']
                }
            }
        },
        // Tauri supports es2021
        target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari14",
        // 启用压缩优化
        minify: !process.env.TAURI_DEBUG ? "terser" : false,
        terserOptions: {
            compress: {
                drop_console: true,
                drop_debugger: true
            }
        },
        // produce sourcemaps for debug builds
        sourcemap: !!process.env.TAURI_DEBUG,
        // 设置chunk大小警告限制
        chunkSizeWarningLimit: 1000,
        // 启用CSS代码分割
        cssCodeSplit: true
    }
});
