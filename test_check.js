const puppeteer = require('puppeteer');
(async () => {
    const browser = await puppeteer.launch({ headless: 'new' });
    try {
        const page = await browser.newPage();
        let hasError = false;
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
        await page.goto('https://uat.buildwithruud.com/admin', { waitUntil: 'networkidle0', timeout: 15000 });
        await new Promise(r => setTimeout(r, 5000));
        
        // check revision
        const revision = await page.evaluate(() => {
            // Usually found in a meta tag or html data attribute
            const html = document.querySelector('html');
            return html ? html.getAttribute('data-revision') : null;
        });
        console.log("Deployed Revision on client:", revision);
        
        if (hasError) console.log("-> Still panicking.");
        else console.log("-> NO PANIC.");
    } catch (e) {
        console.log("-> error:", e);
    }
    await browser.close();
})();
