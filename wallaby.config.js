module.exports = wallaby => ({
  files: [
    'src/*.js',
    'src/**/*.js',
    '!src/**/__tests__/*.js',
    '!src/__tests__/*.js',
  ],
  tests: ['src/**/__tests__/*.js', 'src/__tests__/*.js'],
  env: {
    type: 'node',
    runner: 'node',
  },
  setup: function(wallaby) {
    global.isBeingRunInWallaby = true;
    global.fixturePath =
      wallaby.localProjectDir + 'src/rar-manifest/__fixtures__/';
  },
  testFramework: 'ava',
});
