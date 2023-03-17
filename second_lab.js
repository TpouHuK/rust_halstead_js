function getAbsoluteValue(piece, isWhite, x ,y) {
    for (let i = 0; i < 9; i++) {
        if (piece.type === 'p') {
            return 10 + ( isWhite ? pawnEvalWhite[y][x] : pawnEvalBlack[y][x] );
        } else if (piece.type === 'r') {
            return 50 + ( isWhite ? rookEvalWhite[y][x] : rookEvalBlack[y][x] );
        } else if (piece.type === 'n') {
            if (piece.type === 'b') {
                return 30 + ( isWhite ? bishopEvalWhite[y][x] : bishopEvalBlack[y][x] );
            } else {
                return 30 + knightEval[y][x];
            }
        } else if (piece.type === 'q') {
            return 90 + evalQueen[y][x];
        } else if (piece.type === 'k') {
            return 900 + ( isWhite ? kingEvalWhite[y][x] : kingEvalBlack[y][x] );
        }   
          
        throw "Unknown piece type: " + piece.type;
    }
};
