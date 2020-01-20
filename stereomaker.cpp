//algorithm was taken from http://www.techmind.org/stereo/stech.html
#include "stereomaker.h"
#include <QPainter>
#include <QTime>


QVector<QRgb> StereoMaker::grayscale;

StereoMaker::StereoMaker()
{
    depthsep=(int*)malloc(256*sizeof(int));
}

void StereoMaker::composeDepth(QImage & depth,QImage & compose)
{
    int dw=depth.width(),dh=depth.height(),cw=compose.width(),ch=compose.height();
    uchar * cptr;
    uchar * dptr;

    cptr=compose.scanLine(0);
    dptr=depth.scanLine(0);
    composeDepthGeneric(dptr, dw, dh, cptr, cw, ch);

}

const QVector<QRgb> & StereoMaker::getGrayScale()
{
    if (StereoMaker::grayscale.isEmpty())
    {
        int colorsetter;
        for (colorsetter = 0 ; colorsetter < 256 ; colorsetter++)
            StereoMaker::grayscale.append(qRgb(colorsetter,colorsetter,colorsetter));
    }
    return StereoMaker::grayscale;
}

QImage StereoMaker::render(const QImage & map,const QImage & ptrn,Preset *psettings,QProgressBar * qpbar,const QImage * eye_helper_right,const QImage * eye_helper_left,bool show_helper)
{
    qpbar->setMinimum(0);
    qpbar->setMaximum(10);
    qpbar->setValue(0);

    int width = map.width();
    int height = map.height();

    int oversam = 6;
//    int vwidth = oversam*width*2;

    int dpi = psettings->getDotsPerInch();
    int observer_distance = dpi * psettings->getObserverDistance();
    int eye_separation = dpi * psettings->getEyeSeperation();
    int max_depth=dpi*psettings->getMaximumDepth();
    int vmaxsep=(int)(((long)oversam*eye_separation*max_depth*2)/(max_depth+observer_distance));
    int maxsep=vmaxsep/(oversam*2); // pattern must be at least this wide

    int progbarval=0;
    //benchmark
    QTime t_time;
    t_time.start();
    //QImage pattern=ptrn.scaled(vmaxsep/oversam+1,(ptrn.height()*(maxsep+1))/ptrn.width(),Qt::IgnoreAspectRatio,Qt::SmoothTransformation);
    QImage pattern=ptrn.convertToFormat(QImage::Format_RGBA8888);
    QImage result(map.width(), map.height(), pattern.format());
    renderFromMap(map.scanLine(0), width, height, max_depth, dpi * psettings->getMinimumDepth(), pattern.scanLine(0), pattern.width(), pattern.height(), result.scanLine(0), dpi, observer_distance, eye_separation);
//        if (progbarval!=10*y/(height-1))
//        {
//            progbarval=10*y/(height-1);
//            qpbar->setValue(progbarval);
//        }
    qDebug("Time elapsed: %d ms", t_time.elapsed());
    if (show_helper)
    {
        QPainter painter(&result);
        int rw;
        if (eye_helper_left==0)
            rw=15*psettings->getDotsPerInch()/72;
        else
            rw=eye_helper_left->width();
        int rect_sep;
        if(!psettings->getIsParallel())
        {
            int mindepth=dpi*psettings->getMinimumDepth();
            rect_sep=(int)(((long)eye_separation*mindepth)/(mindepth+observer_distance));
        }
        else
            rect_sep=maxsep;
        if (eye_helper_left!=0)
        {
            painter.drawImage(QPoint(result.width()/2-rect_sep/2-rw/2,5),*eye_helper_left);
            painter.drawImage(QPoint(result.width()/2+rect_sep/2-rw/2,5),*eye_helper_right);

        }
        else
        {
            painter.setPen(QPen(QColor(255,255,255,128)));
            painter.drawRect( result.width()/2-rect_sep/2-rw/2, 5, rw, rw );
            painter.drawRect( result.width()/2+rect_sep/2-rw/2, 5, rw, rw );
            painter.setPen(QPen(QColor(0,0,0,128)));
            painter.drawRect( result.width()/2-rect_sep/2-rw/2+1, 5+1, rw-2, rw-2 );
            painter.drawRect( result.width()/2+rect_sep/2-rw/2+1, 5+1, rw-2, rw-2 );

            painter.fillRect( result.width()/2-rect_sep/2-rw/2+2, 5+2, rw-3, rw-3 ,QColor(255,255,255,60));
            painter.fillRect( result.width()/2+rect_sep/2-rw/2+2, 5+2, rw-3, rw-3 ,QColor(255,255,255,60));
        }
    }
    return result;
}

//QImage StereoMaker::render(const QImage & map,const QImage & ptrn,Preset *psettings,QProgressBar * qpbar,const QImage * eye_helper_right,const QImage * eye_helper_left,bool show_helper)
//{
//    qpbar->setMinimum(0);
//    qpbar->setMaximum(10);
//    qpbar->setValue(0);
//    int width = map.width();
//    int height = map.height();

//    int oversam = 6;
//    int vwidth = oversam*width*2;

//    int x, left;
//    int y, right;
//    int *lookL =(int*) malloc(vwidth*sizeof(int));
//    int *lookR =(int*) malloc(vwidth*sizeof(int));

//    int dpi = psettings->getDotsPerInch();
//    int yShift=dpi/16;
//    int obsDist;
//    obsDist=dpi*psettings->getObserverDistance();
//    int eyeSep;
//    eyeSep=dpi*psettings->getEyeSeperation();
//    int veyeSep=eyeSep*oversam*2;

//    int maxdepth=dpi*psettings->getMaximumDepth();
//    int vmaxsep=(int)(((long)oversam*eyeSep*maxdepth*2)/(maxdepth+obsDist));
//    int maxsep=vmaxsep/(oversam*2); // pattern must be at least this wide


//    QImage pattern=ptrn.scaled(vmaxsep/oversam+1,(ptrn.height()*(maxsep+1))/ptrn.width(),Qt::IgnoreAspectRatio,Qt::SmoothTransformation);
//    if (pattern.depth()!=32)
//        pattern=pattern.convertToFormat(QImage::Format_ARGB32);
//    QImage result(width,height,pattern.format());
//    QImage CurResultLine(vwidth,1,pattern.format());
//    QImage CurResultScaledLine;//scaled down

//    int s=vwidth/2-vmaxsep/2;
//    int poffset=vmaxsep-(s % vmaxsep);
//    int pattern_height=pattern.height();


//    int featureZ,sep=0;
//    bool visual;
//    int lastlinked;
//    int progbarval=0;
//    int maxheight=dpi*(psettings->getMaximumDepth()-psettings->getMinimumDepth());
//    //benchmark
//    QTime t_time;
//    t_time.start();
//    unsigned int **patternptr=(unsigned int **)malloc(pattern_height*sizeof(void*));
//    for (int i=pattern_height-1;i>=0;i--)
//    {
//        patternptr[i]=(unsigned int*)pattern.scanLine(i);
//    }

//    for(int depth_index=0;depth_index<256;depth_index++)
//    {
//        featureZ=maxdepth-depth_index*maxheight/256;
//        depthsep[depth_index]=(int)((veyeSep*featureZ)/(featureZ+obsDist));
//    }

//    uint *dataptr,*lineptr;
//    uchar * mapptr=(uchar *)malloc(map.width()*2);
//    for (y=0;y<height;y++)
//    {
//        for (x=0; x<vwidth; x++)
//        {
//            lookL[x]=x;
//            lookR[x]=x;
//        }
//        scaleLine(mapptr,map.scanLine(y),width);
//        for (x=0; x<vwidth; x++)
//        {
//            if (x%oversam==0)
//            {
//                sep=depthsep[mapptr[x/oversam]];
//            }

//            left=x-sep/2; right=left+sep;
//            visual=true;
//            if ((left>=0) && (right<vwidth))
//            {
//                if (lookL[right]!=right) // right pt already linked
//                {
//                    if (lookL[right]<left) // deeper than current
//                    {
//                        lookR[lookL[right]]=lookL[right]; // break old links
//                        lookL[right]=right;
//                    }
//                    else visual=false;
//                }
//                if (lookR[left]!=left) // left pt already linked
//                {
//                    if (lookR[left]>right) // deeper than current
//                    {
//                        lookL[lookR[left]]=lookR[left]; // break old links
//                        lookR[left]=left;
//                    }
//                    else visual=false;
//                }
//                if (visual)
//                {
//                    lookL[right]=left;
//                    lookR[left]=right;
//                } // make link
//            }
//        }



//        lastlinked=-10; // dummy initial value

//        lineptr=(uint*)CurResultLine.scanLine(0);
//        dataptr=lineptr+s;
//        for (x=s; x<vwidth; x++,dataptr++)
//        {
//            if ((lookL[x]==x) || (lookL[x]<s))
//            {
//                if (lastlinked==(x-1))
//                    *dataptr=*(dataptr-1);
//                else
//                {
//                    *dataptr=patternptr[(y+((x-s)/vmaxsep)*yShift+pattern_height) % pattern_height][((x+poffset) % vmaxsep)/oversam];
//                }
//            }
//            else
//            {
//                *dataptr=*(lineptr+lookL[x]);
//                lastlinked=x; // keep track of the last pixel to be constrained
//            }
//        }

//        lastlinked=-10; // dummy initial value
//        dataptr=lineptr+s-1;
//        for (x=s-1; x>=0; x--,dataptr--)
//            {
//            if (lookR[x]==x)
//            {
//                if (lastlinked==(x+1))
//                    *dataptr=*(dataptr+1);
//                else
//                {
//                    *dataptr=patternptr[(y+((s-x)/vmaxsep+1)*yShift+pattern_height) % pattern_height][((x+poffset) % vmaxsep)/oversam];
//                }
//            }
//            else
//            {
//                *dataptr=*(lineptr+lookR[x]);
//                lastlinked=x; // keep track of the last pixel to be constrained
//            }
//        }


//        CurResultScaledLine = CurResultLine.scaledToWidth(width,Qt::SmoothTransformation);
//        memcpy(result.scanLine(y),CurResultScaledLine.scanLine(0),result.bytesPerLine());
//        if (progbarval!=10*y/(height-1))
//        {
//            progbarval=10*y/(height-1);
//            qpbar->setValue(progbarval);
//        }
//    }
//    qDebug("Time elapsed: %d ms", t_time.elapsed());
//    free(lookL);
//    free(lookR);
//    free(mapptr);
//    free(patternptr);
//    if (show_helper)
//    {
//        QPainter painter(&result);
//        int rw;
//        if (eye_helper_left==0)
//            rw=15*psettings->getDotsPerInch()/72;
//        else
//            rw=eye_helper_left->width();
//        int rect_sep;
//        if(!psettings->getIsParallel())
//        {
//            int mindepth=dpi*psettings->getMinimumDepth();
//            rect_sep=(int)(((long)eyeSep*mindepth)/(mindepth+obsDist));
//        }
//        else
//            rect_sep=maxsep;
//        if (eye_helper_left!=0)
//        {
//            painter.drawImage(QPoint(result.width()/2-rect_sep/2-rw/2,5),*eye_helper_left);
//            painter.drawImage(QPoint(result.width()/2+rect_sep/2-rw/2,5),*eye_helper_right);

//        }
//        else
//        {
//            painter.setPen(QPen(QColor(255,255,255,128)));
//            painter.drawRect( result.width()/2-rect_sep/2-rw/2, 5, rw, rw );
//            painter.drawRect( result.width()/2+rect_sep/2-rw/2, 5, rw, rw );
//            painter.setPen(QPen(QColor(0,0,0,128)));
//            painter.drawRect( result.width()/2-rect_sep/2-rw/2+1, 5+1, rw-2, rw-2 );
//            painter.drawRect( result.width()/2+rect_sep/2-rw/2+1, 5+1, rw-2, rw-2 );

//            painter.fillRect( result.width()/2-rect_sep/2-rw/2+2, 5+2, rw-3, rw-3 ,QColor(255,255,255,60));
//            painter.fillRect( result.width()/2+rect_sep/2-rw/2+2, 5+2, rw-3, rw-3 ,QColor(255,255,255,60));
//        }
//    }
//    return result;
//}
