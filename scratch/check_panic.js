const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ args: ['--no-sandbox', '--disable-setuid-sandbox'] });
  const page = await browser.newPage();
  
  page.on('console', msg => {
    if (msg.type() === 'error' || msg.text().includes('panicked')) {
        console.log(`PAGE LOG: ${msg.text()}`);
    }
  });
  
  page.on('pageerror', error => {
    console.log(`PAGE ERROR: ${error.message}`);
  });

  try {
    await page.goto('https://uat.buildwithruud.com/admin', { waitUntil: 'networkidle2' });
  } catch (e) {
    console.log(`GOTO ERROR: ${e.message}`);
  }
  
  await browser.close();
})();
