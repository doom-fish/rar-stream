module.exports = function (wallaby) {
  return {
    files: [
      'src/**/*.js',
      '!src/**/__tests__/*.js'
    ],

    tests: [
      'src/**/__tests__/*.js'
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
      require('babel-polyfill');
    },

    debug: true
  };
};
