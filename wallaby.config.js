var which = require('which');
module.exports = function(wallaby) {
    return {
        files: [
            'src/**/*.js',
            '!src/**/__tests__/*.js',
            '!node_modules/**/*.js'
        ],
        tests: [
            'src/**/__tests__/*.js',
            '!src/rar-manifest/__tests__/*.js',
            '!node_modules/**/*.js'
        ],
        env: {
            type: 'node',
            runner: which.sync('node')
        },
        recycle: true,
        compilers: {
            '**/*.js': wallaby.compilers.babel()
        },
        testFramework: 'ava',
        setup: function(wallaby) {
            global.isBeingRunInWallaby = true;
            global.fixturePath = wallaby.localProjectDir +
                'src/rar-manifest/__fixtures__/';
            require('babel-polyfill');
        },
        debug: true
    };
};
