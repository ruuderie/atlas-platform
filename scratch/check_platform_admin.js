const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ 
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--ignore-certificate-errors'],
    ignoreHTTPSErrors: true
  });
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
    await page.goto('https://uat.atlas.oply.co', { waitUntil: 'networkidle2' });
  } catch (e) {
    console.log(`GOTO ERROR: ${e.message}`);
  }
  
  await browser.close();
})();
