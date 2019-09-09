import PyPDF2

def archive(doc, file):
    f = open(file, 'rb')
    pdf = PyPDF2.PdfFileReader(f)
    docinfo = pdf.getDocumentInfo();

    doc["format"] = "pdf"

    doc["author"] = docinfo.author
    doc["creator"] = docinfo.creator
    doc["producer"] = docinfo.producer
    doc["subject"] = docinfo.subject
    doc["title"] = docinfo.title

    doc["pagecount"] = pdf.getNumPages()
