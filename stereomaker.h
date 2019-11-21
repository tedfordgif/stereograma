#ifndef STEREOMAKER_H
#define STEREOMAKER_H

#include <QImage>
#include <QProgressBar>
#include "preset.h"
#include "libstereograma.h"

//void scaleLine(uchar* big,const uchar *original,int sizeoriginal);
class StereoMaker
{
public:
    StereoMaker();
    QImage render(const QImage & dmap,const QImage & ptrn,Preset *psettings,QProgressBar * qpbar,const QImage * eye_helper_right,const QImage * eye_helper_left,bool show_helper);
    void composeDepth(QImage & depth,QImage & compose);
    static const QVector<QRgb> & getGrayScale();
private:
    static QVector<QRgb> grayscale;
    int *depthsep;
};

#endif // STEREOMAKER_H
