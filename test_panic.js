const puppeteer = require('puppeteer');

(async () => {
    const browser = await puppeteer.launch({ headless: 'new' });
    const page = await browser.newPage();
    
    page.on('console', msg => {
        if (msg.type() === 'error' || msg.text().includes('panicked')) {
            console.log('BROWSER LOG:', msg.text());
        }
    });

    page.on('pageerror', error => {
        console.log('PAGE ERROR:', error.message);
    });

    try {
        await page.goto('https://uat.buildwithruud.com/admin', { waitUntil: 'networkidle0', timeout: 15000 });
        await new Promise(r => setTimeout(r, 5000));
        console.log("Done waiting.");
    } catch (e) {
        console.log("Navigation ended:", e.message);
    }
    
    await browser.close();
})();
