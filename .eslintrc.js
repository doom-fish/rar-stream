module.exports = {
    env: {
        es6: true,
        node: true
    },
    parser: 'babel-eslint',
    plugins: ['prettier', 'babel', 'flowtype', 'filenames'],
    extends: 'eslint:recommended',
    parserOptions: {
        sourceType: 'module'
    },
    rules: {
        indent: ['error', 4],
        'linebreak-style': ['error', 'unix'],
        quotes: ['error', 'single'],
        semi: ['error', 'always']
    }
};
