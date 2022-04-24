extern "C"
{
#include <libavutil/avutil.h>
#include <libavutil/opt.h>
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswscale/swscale.h>
}

namespace
{
    template <typename T>
    struct ScopeGuard
    {
        ScopeGuard(T func) : func(func) {}
        ~ScopeGuard() { func(); }
        void discard() { func = nullptr; }
        T func;
    };

    struct UgoiraFrame
    {
        uint16_t delay;
    };

    /* Stein's binary GCD algorithm */
    /* Find greatest common denumerator without recursion */
    uint16_t gcd(uint16_t a, uint16_t b)
    {
        uint16_t r = a - a;
        while (b != 0)
        {
            r = a % b;
            a = b;
            b = r;
        }
        return a;
    }

    float ugoira_cal_fps(const UgoiraFrame *frames, size_t frame_count)
    {
        uint16_t re = frames->delay;
        for (size_t i = 1; i < frame_count; i++)
            re = gcd(re, frames[i].delay);
        return FFMIN(1000 / ((float)re), 60.0f);
    }

    using ReadFuncProto = int (*)(void *opaque, uint8_t *buf, int buf_size);
    using NextFuncProto = void (*)(void *opaque);
    using WriteFuncProto = int (*)(void *opaque, uint8_t *buf, int buf_size);
    using SeekFuncProto = int64_t (*)(void *opaque, int64_t offset, int whence);

#define CHECK(exp)                                                           \
    do                                                                       \
    {                                                                        \
        const auto ret = (exp);                                              \
        if (ret < 0)                                                         \
        {                                                                    \
            printf("Error: %d: %s:%d: %s\n", ret, __FILE__, __LINE__, #exp); \
            return -1;                                                       \
        }                                                                    \
    } while (0)

    int encode_video(AVFrame *ofr, AVPacket *pkt, AVFormatContext *oc, AVCodecContext *eoc, int64_t *pts, unsigned int stream_index, AVRational time_base)
    {
        if (ofr != nullptr)
        {
            ofr->pts = *pts;
            *pts += av_rescale_q_rnd(1, time_base, oc->streams[stream_index]->time_base, static_cast<AVRounding>(AV_ROUND_NEAR_INF | AV_ROUND_PASS_MINMAX));
            ofr->pkt_dts = ofr->pts;
        }
        auto ret = avcodec_send_frame(eoc, ofr);
        if (ret < 0 && ret != AVERROR_EOF)
        {
            printf("Error: %d: %s:%d: avcodec_send_frame\n", ret, __FILE__, __LINE__);
            return false;
        }

        ret = avcodec_receive_packet(eoc, pkt);
        if (ret == AVERROR_EOF || ret == AVERROR(EAGAIN))
        {
            return false;
        }

        pkt->stream_index = stream_index;
        CHECK(av_write_frame(oc, pkt));

        return true;
    }
}

extern "C" int convert(
    void *opaque,
    ReadFuncProto read,
    NextFuncProto next,
    WriteFuncProto write,
    SeekFuncProto seek,
    const UgoiraFrame *frames,
    size_t frame_count)
{
    av_log_set_level(AV_LOG_ERROR);

    const auto fps = ugoira_cal_fps(frames, frame_count);
    const auto framerate = AVRational{int(fps * AV_TIME_BASE + 0.5f), AV_TIME_BASE};
    const auto time_base = AVRational{framerate.den, framerate.num};

    /* Prepare output */
    auto output_codec = avcodec_find_encoder_by_name("libx264");
    auto oc = (AVFormatContext *)nullptr;
    CHECK(avformat_alloc_output_context2(&oc, nullptr, "mp4", nullptr));
    auto _oc_guard = ScopeGuard([&]()
                                { avformat_free_context(oc); });

    auto buffer = (unsigned char *)av_malloc(0x4000);
    auto oioc = avio_alloc_context(buffer, 0x4000, 1, opaque, nullptr, write, seek);
    auto _oioc_guard = ScopeGuard([&]()
                                  { av_freep(&oioc->buffer);
                                         avio_context_free(&oioc); });

    oc->pb = oioc;
    oc->flags |= AVFMT_FLAG_CUSTOM_IO;

    auto os = avformat_new_stream(oc, output_codec);
    os->avg_frame_rate = framerate;
    os->r_frame_rate = framerate;
    os->time_base = AVRational{1, AV_TIME_BASE};

    auto eoc = avcodec_alloc_context3(output_codec);
    auto _eoc_guard = ScopeGuard([&]()
                                 { avcodec_free_context(&eoc); });

    auto ifr = av_frame_alloc();
    auto _ifr_guard = ScopeGuard([&]()
                                 { av_frame_free(&ifr); });

    auto ofr = av_frame_alloc();
    auto _ofr_guard = ScopeGuard([&]()
                                 { av_frame_free(&ofr); });

    auto pkt = av_packet_alloc();
    auto _pkt_guard = ScopeGuard([&]()
                                 { av_packet_free(&pkt); });

    auto sws_ctx = (SwsContext *)nullptr;
    auto _sws_ctx_guard = ScopeGuard([&]()
                                     { sws_freeContext(sws_ctx); });

    const auto UgoiraTimeBase = AVRational{1, 1000};
    int64_t pts = 0, max_de = 0;
    auto pre_pixfmt = AV_PIX_FMT_NONE;
    auto pre_width = -1, pre_height = -1;

    for (size_t i = 0; i < frame_count; ++i)
    {
        /* Open input file */
        auto ic = avformat_alloc_context();
        auto _ic_guard = ScopeGuard([&]()
                                    { avformat_free_context(ic); });

        next(opaque);

        auto buffer = (unsigned char *)av_malloc(0x4000);
        auto iioc = avio_alloc_context(buffer, 0x4000, 0, opaque, read, nullptr, nullptr);
        auto _iioc_guard = ScopeGuard([&]()
                                      { av_freep(&iioc->buffer);
                                         avio_context_free(&iioc); });

        ic->pb = iioc;
        ic->flags |= AVFMT_FLAG_CUSTOM_IO;

        CHECK(avformat_open_input(&ic, nullptr, nullptr, nullptr));
        auto _ic_open_guard = ScopeGuard([&]()
                                         { avformat_close_input(&ic); });

        CHECK(avformat_find_stream_info(ic, nullptr));

        auto is = ic->streams[0];

        auto input_codec = avcodec_find_decoder(is->codecpar->codec_id);
        auto eic = avcodec_alloc_context3(input_codec);
        CHECK(avcodec_parameters_to_context(eic, is->codecpar));
        CHECK(avcodec_open2(eic, input_codec, nullptr));
        auto _eic_guard = ScopeGuard([&]()
                                     { avcodec_free_context(&eic); });

        /* Prepare output on the first frame */
        if (i == 0)
        {
            eoc->width = (eic->width + 1) & ~1;
            eoc->height = (eic->height + 1) & ~1;
            eoc->sample_aspect_ratio = eic->sample_aspect_ratio;
            eoc->framerate = framerate;
            eoc->time_base = AVRational{1, AV_TIME_BASE};
            eoc->pix_fmt = AV_PIX_FMT_YUV420P;

            CHECK(av_opt_set(eoc->priv_data, "preset", "slow", 0));
            CHECK(av_opt_set_int(eoc->priv_data, "crf", 18, 0));
            CHECK(av_opt_set(eoc->priv_data, "level", "2", 0));
            CHECK(av_opt_set(eoc->priv_data, "profile", "main", 0));

            ofr->width = eoc->width;
            ofr->height = eoc->height;
            ofr->format = eoc->pix_fmt;
            CHECK(av_frame_get_buffer(ofr, 0));

            CHECK(avcodec_open2(eoc, output_codec, nullptr));
            CHECK(avcodec_parameters_from_context(os->codecpar, eoc));

            CHECK(avformat_write_header(oc, nullptr));
        }

        /* Create software scaler if needed */
        if (!sws_ctx || eic->pix_fmt != pre_pixfmt || eic->width != pre_width || eic->height != pre_height)
        {
            if (sws_ctx)
            {
                sws_freeContext(sws_ctx);
                sws_ctx = NULL;
            }
            if (!(sws_ctx = sws_getContext(eic->width, eic->height, eic->pix_fmt, eoc->width, eoc->height, eoc->pix_fmt, SWS_BILINEAR, NULL, NULL, NULL)))
            {
                exit(1);
            }
            pre_pixfmt = eic->pix_fmt;
            pre_width = eic->width;
            pre_height = eic->height;
        }

        CHECK(av_read_frame(ic, pkt));
        CHECK(avcodec_send_packet(eic, pkt));
        av_packet_unref(pkt);

        auto err = (avcodec_receive_frame(eic, ifr));
        if (err >= 0)
        {
            err = 0;
            if ((err = av_frame_make_writable(ofr)) < 0)
            {
                printf("av_frame_make_writable failed\n");
                exit(1);
            }
            if ((err = sws_scale(sws_ctx, (const uint8_t *const *)ifr->data, ifr->linesize, 0, ifr->height, ofr->data, ofr->linesize)) < 0)
            {
                printf("sws_scale failed\n");
                exit(1);
            }
            err = 0;
        }
        else if (err == AVERROR(EAGAIN))
        {
            err = 0;
            continue;
        }
        else
        {
            printf("avcodec_receive_frame failed\n");
            exit(1);
        }

        max_de += av_rescale_q_rnd(frames[i].delay, UgoiraTimeBase, os->time_base, static_cast<AVRounding>(AV_ROUND_NEAR_INF | AV_ROUND_PASS_MINMAX));

        while (pts < max_de)
        {
            encode_video(ofr, pkt, oc, eoc, &pts, os->index, time_base);
        }

        if (i == frame_count - 1)
        {
            /* Flush the encoder */
            while (encode_video(nullptr, pkt, oc, eoc, nullptr, os->index, time_base))
                ;
        }
    }

    CHECK(av_write_trailer(oc));

    return 0;
}
