import { chromium } from '@playwright/test';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

// Simple arg parser
const args = process.argv.slice(2);
const getArg = (name) => {
    const index = args.indexOf(name);
    return index !== -1 ? args[index + 1] : null;
};

const scriptPath = getArg('--script');
const walletJson = getArg('--wallet');
const proxyUrl = getArg('--proxy');

if (!scriptPath || !walletJson) {
    console.error('Usage: node playwright-runner.js --script <path> --wallet <json> [--proxy <url>]');
    process.exit(1);
}

const wallet = JSON.parse(walletJson);

async function main() {
    console.log(`[Runner] Starting task for wallet: ${wallet.address || 'Unknown'}`);

    const launchOptions = {
        headless: false, // User usually wants to see it, or we make it configurable. 
        // For automation "background", headless=true is better, but for "Browser Automation" card, maybe visible?
        // Let's default to false (visible) so they can see it working, or make it an option.
        args: [
            '--disable-blink-features=AutomationControlled',
            '--no-sandbox',
            '--disable-infobars',
        ]
    };

    if (proxyUrl) {
        launchOptions.proxy = { server: proxyUrl };
    }

    const browser = await chromium.launch(launchOptions);

    // Anti-detection Context Setup
    const context = await browser.newContext({
        userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        viewport: { width: 1280, height: 800 },
        deviceScaleFactor: 1,
        locale: 'en-US',
        timezoneId: 'America/New_York',
        permissions: ['geolocation', 'notifications'],
    });

    // Evasion Scripts
    await context.addInitScript(() => {
        Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
    });

    try {
        // Dynamic Import of User Script
        // Convert path to file URL for Windows compatibility
        const scriptUrl =  path.isAbsolute(scriptPath) ?  `file://${scriptPath}` : `file://${path.resolve(process.cwd(), scriptPath)}`;
        
        console.log(`[Runner] Loading script from: ${scriptUrl}`);
        const userModule = await import(scriptUrl);

        if (typeof userModule.default !== 'function' && typeof userModule.run !== 'function') {
            throw new Error('Script must export a default function or a named "run" function.');
        }

        const runFunc = userModule.default || userModule.run;

        // Execute User Logic
        console.log('[Runner] Executing user script...');
        await runFunc({ 
            browser, 
            context, 
            page: await context.newPage(), 
            wallet 
        });
        console.log('[Runner] Execution completed successfully.');

    } catch (error) {
        console.error('[Runner] Error during execution:', error);
        process.exit(1);
    } finally {
        await context.close();
        await browser.close();
    }
}

main();
