const express = require('express');
const { exec } = require('child_process');
const path = require('path');
const app = express();
const PORT = 8081;

app.use(express.json());
app.use(express.static(path.join(__dirname)));

// CLI command runner
app.post('/run-cmd', (req, res) => {
  const { cmd } = req.body;
  if (!cmd) {
    return res.json({ error: 'No command provided' });
  }

  const cliPath = '/home/cliff/kaspa-graffiti/target/release/kaspa-graffiti-cli';
  const fullCmd = `${cliPath} ${cmd}`;
  
  console.log('Executing:', fullCmd);
  
  exec(fullCmd, { timeout: 30000 }, (error, stdout, stderr) => {
    if (error) {
      res.json({ error: stderr || error.message });
      return;
    }
    try {
      const json = JSON.parse(stdout);
      res.json(json);
    } catch (e) {
      res.json({ output: stdout, stderr: stderr });
    }
  });
});

// Serve index.html
app.get('/', (req, res) => {
  res.sendFile(path.join(__dirname, 'index.html'));
});

app.listen(PORT, () => {
  console.log(`Kaspa CLI Test Console: http://localhost:${PORT}`);
  console.log(`CLI Path: /home/cliff/kaspa-graffiti/target/release/kaspa-graffiti-cli`);
});
