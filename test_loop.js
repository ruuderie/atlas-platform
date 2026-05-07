const puppeteer = require('puppeteer');

(async () => {
    while (true) {
        let hasError = false;
        const browser = await puppeteer.launch({ headless: 'new' });
        try {
            const page = await browser.newPage();
            
            page.on('console', msg => {
                if (msg.type() === 'error' || msg.text().includes('panicked')) {
                    console.log('BROWSER LOG:', msg.text());
                    hasError = true;
                }
            });

            page.on('pageerror', error => {
                console.log('PAGE ERROR:', error.message);
                hasError = true;
            });

            console.log("Checking https://uat.buildwithruud.com/admin...");
            await page.goto('https://uat.buildwithruud.com/admin', { waitUntil: 'networkidle0', timeout: 15000 });
            await new Promise(r => setTimeout(r, 3000));
            
            if (hasError) {
                console.log("-> Still panicking.");
            } else {
                console.log("-> NO PANIC! Deployment might be live!");
                await browser.close();
                break;
            }
        } catch (e) {
            console.log("-> Navigation failed:", e.message);
        }
        await browser.close();
        
        console.log("Waiting 15 seconds before next check...");
        await new Promise(r => setTimeout(r, 15000));
    }
})();
