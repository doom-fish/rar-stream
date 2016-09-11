module.exports = function (wallaby) {
  return {
    files: [
      'src/**/*.js',
      '!src/**/__tests__/*.js',
      '!node_modules/**/*.js'
    ],

    tests: [
      'src/**/__tests__/*.js',
      '!node_modules/**/*.js'
    ],

    env: {
      type: 'node',
      runner: '/usr/local/bin/node'
    },

    compilers: {
      '**/*.js': wallaby.compilers.babel()
    },

    testFramework: 'ava',

    setup: function () {
      require('babel-polyfill')
    },

    debug: true
  };
};
